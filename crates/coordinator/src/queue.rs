use common::{
    job::{
        JobResult, 
        JobStatus, 
        Priority,
        Job, 
    }, 
    message::{
        GetJobStatusResponse, 
        WorkerHeartbeat, 
        WorkerStatus,
        WorkerInfo, 
    }
};
use cron::Schedule;
use std::{collections::{
    HashMap, VecDeque 
}, str::FromStr};
use chrono::{
    Duration, 
    Utc
};
use rusqlite::Connection;
use uuid::Uuid;

use crate::db;

pub struct JobQueue {
    jobs: HashMap<Uuid, Job>,
    schedules: HashMap<Uuid, Job>,
    results: HashMap<Uuid, JobResult>,

    pending_high: VecDeque<Job>,
    pending_medium: VecDeque<Job>,
    pending_low: VecDeque<Job>,

    workers: HashMap<Uuid, WorkerInfo>,
    connection: Connection
}

impl JobQueue {
    pub fn new() -> Self {
        let mut queue = JobQueue {
            jobs: HashMap::new(), 
            schedules: HashMap::new(),
            results: HashMap::new(),

            pending_high: VecDeque::new(), 
            pending_medium: VecDeque::new(), 
            pending_low: VecDeque::new(),

            workers: HashMap::new(),
            connection: Connection::open("scheduler.db").unwrap()
        };

        let jobs = db::load_pending_jobs(&queue.connection).unwrap();
        
        for job in jobs {
            if job.is_recurring.unwrap() {
                queue.schedules.insert(job.id, job.clone());
                log::info!("Loading schedule into HashMap: id={}, schedule={:?}", job.id, job.schedule);
            } else {
                queue.jobs.insert(job.id, job.clone());

                match job.priority {
                    Priority::HIGH => queue.pending_high.push_back(job),
                    Priority::MEDIUM => queue.pending_medium.push_back(job),
                    Priority::LOW => queue.pending_low.push_back(job),
                }
                }
        }

        queue
    }

    // Worker Functions

    pub fn register_worker(&mut self, info: WorkerInfo) {
        self.workers.insert(info.worker_id, info.clone());
    }

    pub fn update_worker_heartbeat(&mut self, heartbeat: WorkerHeartbeat) {
        if let Some(worker) = self.workers.get_mut(&heartbeat.worker_id) {
            worker.last_seen = heartbeat.timestamp;

            if worker.status == WorkerStatus::DEAD {
                worker.status = WorkerStatus::ALIVE
            }
        }
    }

    pub fn check_worker(&mut self) {
        for (worker_id, worker_info) in self.workers.clone() {
            let last_beat = Utc::now() - worker_info.last_seen;
            
            if last_beat > Duration::seconds(60) && worker_info.status == WorkerStatus::ALIVE {
                let recover_job_id = worker_info.current_job_id;

                if let Some(w) = self.workers.get_mut(&worker_id) {
                    w.status = WorkerStatus::DEAD;
                    w.current_job_id = None;
                }

                if let Some(job_id) = recover_job_id {
                    if let Some(j) = self.get_job(job_id) {
                        match j.priority {
                            Priority::HIGH => self.pending_high.push_back(j.clone()),
                            Priority::MEDIUM => self.pending_medium.push_back(j.clone()),
                            Priority::LOW => self.pending_low.push_back(j.clone()),
                        }
                        self.update_job_status(j.id, JobStatus::PENDING);
                        
                        log::error!("Worker {} is dead, recovered job id: {}", worker_id, job_id);
                    }
                } else {
                    log::warn!("Worker {} heartbeat too old, marking as dead", worker_id);
                }
            } 
        }
    }

    // Job Functions
    
    pub fn add_scheduled_jobs(&mut self, job: Job) {
        db::insert_job(&self.connection, job.clone()).unwrap();
        self.schedules.insert(job.id, job.clone());
    }

    pub fn check_scheduled_jobs(&mut self) {
        log::info!("Checking {} scheduled jobs", self.schedules.len());
        for (job_id, jobs) in self.schedules.clone() {
            if let Some(next_run_time) = jobs.next_run {
                let run_time = next_run_time - Utc::now();

                log::info!("Old run time {}", run_time);

                // +/- 30 seconds window, so ~60 second window
                if Duration::seconds(-30) <= run_time && run_time <= Duration::seconds(30) {
                    let sched_job = Job {
                        id: Uuid::new_v4(),
                        command: jobs.command,
                        args: jobs.args,
                        status: jobs.status,
                        timestamp: Utc::now(),
                        
                        retry_count: 0,
                        max_retries: jobs.max_retries,

                        priority: jobs.priority,

                        parent_schedule_id: Some(job_id),

                        schedule: None,
                        next_run: None,
                        is_recurring: None
                    };


                    self.add_job(sched_job);
                }

                if jobs.is_recurring.unwrap() {
                    if let Some(j) = self.schedules.get_mut(&job_id) {
                        let next = Schedule::from_str(&j.schedule.clone().unwrap())
                            .ok()
                            .and_then(|s| s.upcoming(Utc).next());
                        
                        log::info!("New run time {}", next.unwrap());

                        if let Some(next_time) = next {
                            log::info!("New run time {}", next_time);
                            db::update_schedule_run(&self.connection, job_id, next_time).unwrap();
                            j.next_run = Some(next_time);
                        }
                    }
                }
            }
        }
    }

    pub fn add_job(&mut self, job: Job) {
        db::insert_job(&self.connection, job.clone()).unwrap();
        self.jobs.insert(job.id, job.clone());
        
        match job.priority {
            Priority::HIGH => self.pending_high.push_back(job),
            Priority::MEDIUM => self.pending_medium.push_back(job),
            Priority::LOW => self.pending_low.push_back(job),
        }
    }

    pub fn get_next_job(&mut self, requester: Uuid) -> Option<Job> {
        let job = self.pending_high.pop_front()
            .or_else(|| self.pending_medium.pop_front())
            .or_else(|| self.pending_low.pop_front());
        
        if job.is_some() {
            self.update_job_status(job.clone().unwrap().id, JobStatus::RUNNING);
            
            if let Some(worker) = self.workers.get_mut(&requester) {
                worker.current_job_id = Some(job.clone().unwrap().id);
            }

            db::update_job_status(&self.connection, job.clone().unwrap().id, JobStatus::RUNNING).unwrap();
        }
        
        job
    }

    pub fn get_job(&self, job_id: Uuid) -> Option<Job> {
        if self.jobs.get(&job_id).cloned().is_some() {
            self.jobs.get(&job_id).cloned()
        } else {
            self.schedules.get(&job_id).cloned()
        }
    }

    pub fn get_job_status(&self, job_id: Uuid) -> Option<GetJobStatusResponse> {
        Some(GetJobStatusResponse {
            job: self.get_job(job_id)?,
            result: self.results.get(&job_id).cloned()
        })
    }

    pub fn retry_job(&mut self, job_id: Uuid) {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            job.status = JobStatus::RETRYING;
            job.retry_count += 1;

            db::update_job_status(&self.connection, job_id.clone(), JobStatus::RETRYING).unwrap();
            db::update_retry_count(&self.connection, job_id.clone(), job.retry_count).unwrap();

            match job.priority {
                Priority::HIGH => self.pending_high.push_back(job.clone()),
                Priority::MEDIUM => self.pending_medium.push_back(job.clone()),
                Priority::LOW => self.pending_low.push_back(job.clone()),
            }
        }
    }

    pub fn update_job_status(&mut self, job_id: Uuid, status: JobStatus) {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            db::update_job_status(&self.connection, job_id.clone(), status.clone()).unwrap();
            job.status = status;
        }
    }

    pub fn store_results(&mut self, job_id: Uuid, job_results: JobResult) {
        // update job queue with results
        db::insert_results(&self.connection, job_id.clone(), job_results.clone()).unwrap();
        self.results.insert(job_id, job_results);
    }
}