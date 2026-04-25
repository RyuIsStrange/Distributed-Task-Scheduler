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
}, process::exit, str::FromStr};
use chrono::{
    Duration, 
    Utc
};
use rusqlite::Connection;
use uuid::Uuid;

use crate::{db, metrics};

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
            connection: Connection::open("scheduler.db").unwrap_or_else(|e| {
                log::error!("DB Error: Failed to open database, exiting program.\n Error: {}", e); 
                exit(1); 
            })
        };

        let jobs = db::load_pending_jobs(&queue.connection).unwrap_or_else(|e| {
                log::error!("DB Error: Failed to load pending jobs, exiting program.\n Error: {}", e); 
                exit(1); 
            });
        
        for job in jobs {
            if job.is_recurring {
                queue.schedules.insert(job.id, job.clone());
                log::info!("Loading schedule into HashMap: id={}, schedule={:?}", job.id, job.schedule);
            } else {
                queue.jobs.insert(job.id, job.clone());

                metrics::QUEUE_DEPTH.with_label_values(&[&job.priority.to_string()]).inc();

                match job.priority {
                    Priority::HIGH => queue.pending_high.push_back(job),
                    Priority::MEDIUM => queue.pending_medium.push_back(job),
                    Priority::LOW => queue.pending_low.push_back(job),
                }
                }
        }

        queue
    }

    pub fn queue_size(&self) -> usize {
        self.jobs.len()
    }

    // Worker Functions

    pub fn is_worker_registered(&self, worker_id: Uuid) -> bool {
        self.workers.contains_key(&worker_id)
    }

    pub fn register_worker(&mut self, info: WorkerInfo) {
        metrics::ACTIVE_WORKERS.inc();

        self.workers.insert(info.worker_id, info.clone());
    }

    pub fn update_worker_heartbeat(&mut self, heartbeat: WorkerHeartbeat) {
        if let Some(worker) = self.workers.get_mut(&heartbeat.worker_id) {
            worker.last_seen = heartbeat.timestamp;

            if worker.status == WorkerStatus::DEAD {
                metrics::ACTIVE_WORKERS.inc();

                worker.status = WorkerStatus::ALIVE
            }
        }
    }

    pub fn check_worker(&mut self) {
        let dead_workers: Vec<_> = self.workers.iter()
            .filter_map(|(id, info)| {
                let last_beat = Utc::now() - info.last_seen;

                if last_beat > Duration::seconds(60) && info.status == WorkerStatus::ALIVE {
                    Some((*id, info.current_job_id))
                } else {
                    None
                }
            })
            .collect();

        for (worker_id, recovered_job_option) in dead_workers {
            if let Some(w) = self.workers.get_mut(&worker_id) {
                w.status = WorkerStatus::DEAD;
                w.current_job_id = None;
            }

            metrics::ACTIVE_WORKERS.dec();

            if let Some(job_id) = recovered_job_option {
                if let Some(j) = self.get_job(job_id) {
                    metrics::QUEUE_DEPTH.with_label_values(&[&j.priority.to_string()]).inc();

                    match j.priority {
                        Priority::HIGH => self.pending_high.push_back(j.clone()),
                        Priority::MEDIUM => self.pending_medium.push_back(j.clone()),
                        Priority::LOW => self.pending_low.push_back(j.clone()),
                    }
                    self.update_job_status(j.id, JobStatus::PENDING);
                    
                    log::warn!("Worker {} is dead, recovered job id: {}", worker_id, job_id);
                }
            } else {
                log::warn!("Worker {} heartbeat too old, marking as dead", worker_id);
            }
        }
    }

    // Job Functions
    
    pub fn add_scheduled_jobs(&mut self, job: Job) {
        match db::insert_job(&self.connection, job.clone()) {
            Ok(_) => {},
            Err(err) => {log::error!("DB Error: Failed to insert job into the database for job id: {}\n Error output: {:?}", job.id, err)}
        }

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
                        is_recurring: false,

                        depends_on: None // Might just put the parent ID here as it "depends" on the parent to be running but the parent isn't required for it or smth
                    };


                    self.add_job(sched_job);
                }

                if jobs.is_recurring {
                    if let Some(j) = self.schedules.get_mut(&job_id) {
                        let next;
                        
                        if let Some(sched) = &j.schedule.clone() { 
                            next = Schedule::from_str(sched)
                                .ok()
                                .and_then(|s| s.upcoming(Utc).next());
                            
                            if let Some(next_time) = next {
                                log::info!("New run time {}", next_time);

                                match db::update_schedule_run(&self.connection, job_id, next_time) {
                                    Ok(_) => {},
                                    Err(err) => {log::error!("DB Error: Failed update schedule time for job id: {}\n Error output: {:?}", j.id, err)}   
                                }

                                j.next_run = Some(next_time);
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn add_job(&mut self, job: Job) {
        match db::insert_job(&self.connection, job.clone()) {
            Ok(_) => {},
            Err(err) => {log::error!("DB Error: Failed insert job into database. Job id: {}\n Error output: {:?}", job.id, err)}   
        }
        
        self.jobs.insert(job.id, job.clone());

        metrics::JOBS_SUBMITTED_TOTAL.with_label_values(&[&job.priority.to_string()]).inc();
        metrics::QUEUE_DEPTH.with_label_values(&[&job.priority.to_string()]).inc();
        
        match job.priority {
            Priority::HIGH => self.pending_high.push_back(job),
            Priority::MEDIUM => self.pending_medium.push_back(job),
            Priority::LOW => self.pending_low.push_back(job),
        }
    }

    fn add_worker_job(&mut self, j: Job, requester: Uuid) {
        self.update_job_status(j.id, JobStatus::RUNNING);
        
        if let Some(worker) = self.workers.get_mut(&requester) {
            worker.current_job_id = Some(j.id);
        }

        match db::update_job_status(&self.connection, j.id, JobStatus::RUNNING) {
            Ok(_) => {},
            Err(err) => {log::error!("DB Error: Failed to set job status to running for job id: {}\n Error output: {:?}", j.id, err)}   
        }
    }

    pub fn get_next_job(&mut self, requester: Uuid) -> Option<Job> {
        let job = self.pending_high.pop_front()
            .or_else(|| self.pending_medium.pop_front())
            .or_else(|| self.pending_low.pop_front());
        
        if let Some(j) = job.clone() {
            if let Some(requirements) = j.clone().depends_on && (j.depends_on.iter().len() > 0) {
                let mut completed = vec![];
                let mut failed_req = false;
                
                for id in requirements.iter() {
                    let checking = self.get_job(*id);
                    match checking {
                        Some(job) => {
                            if job.status == JobStatus::COMPLETED {
                                completed.push(id);
                            }
                            if job.status == JobStatus::FAILED || job.status == JobStatus::CANCELED {
                                failed_req = true;
                            }
                        },
                        None => {} // Do nothing so the job gets re-queued as failed to get required job
                    } 
                }

                if completed.len() == requirements.len() {
                    if j.status == JobStatus::WAITING {
                        metrics::JOBS_WAITING_TOTAL.dec();
                    }
                    metrics::QUEUE_DEPTH.with_label_values(&[&j.priority.to_string()]).dec();

                    self.add_worker_job(j.clone(), requester);
                    return Some(j);
                } else {
                    // Add job back into the VecDeque without re-adding it to the DB
                    // As long if one of the required jobs hasn't failed or been canceled
                    if failed_req {
                        metrics::QUEUE_DEPTH.with_label_values(&[&j.priority.to_string()]).dec();

                        self.update_job_status(j.id, JobStatus::FAILED);
                    } else {
                        if j.status != JobStatus::WAITING {
                            metrics::JOBS_WAITING_TOTAL.inc();

                            self.update_job_status(j.id, JobStatus::WAITING);
                        }

                        match j.priority {
                            Priority::HIGH => self.pending_high.push_back(j.clone()),
                            Priority::MEDIUM => self.pending_medium.push_back(j.clone()),
                            Priority::LOW => self.pending_low.push_back(j.clone()),
                        };
                    }

                    return None;
                }

            } else {
                metrics::QUEUE_DEPTH.with_label_values(&[&j.priority.to_string()]).dec();

                self.add_worker_job(j.clone(), requester);
                return Some(j)
            }
        } else {
            return None
        }
    }

    pub fn get_job(&self, job_id: Uuid) -> Option<Job> {
        self.jobs.get(&job_id).cloned().or_else(|| self.schedules.get(&job_id).cloned())
    }

    pub fn get_job_status(&self, job_id: Uuid) -> Option<GetJobStatusResponse> {
        Some(GetJobStatusResponse {
            job: self.get_job(job_id)?,
            result: self.results.get(&job_id).cloned()
        })
    }

    pub fn get_list(&self, status: Option<JobStatus>) -> Result<Vec<Job>, rusqlite::Error> {
        db::get_job_list(&self.connection, status)
    }

    pub fn retry_job(&mut self, job_id: Uuid) {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            job.status = JobStatus::RETRYING;
            job.retry_count += 1;

            match db::update_job_status(&self.connection, job_id, JobStatus::RETRYING) {
                Ok(_) => {},
                Err(err) => {log::error!("DB Error: Failed update status for job id: {}\n Error output: {:?}", job_id, err)}   
            }
            
            match db::update_retry_count(&self.connection, job_id, job.retry_count) {
                Ok(_) => {},
                Err(err) => {log::error!("DB Error: Failed update retry count for job id: {}\n Error output: {:?}", job_id, err)}   
            }

            metrics::QUEUE_DEPTH.with_label_values(&[&job.priority.to_string()]).inc();

            match job.priority {
                Priority::HIGH => self.pending_high.push_back(job.clone()),
                Priority::MEDIUM => self.pending_medium.push_back(job.clone()),
                Priority::LOW => self.pending_low.push_back(job.clone()),
            }
        }
    }

    pub fn update_job_status(&mut self, job_id: Uuid, status: JobStatus) {
        if let Some(job) = self.jobs.get_mut(&job_id) {
            match db::update_job_status(&self.connection, job_id, status.clone()){
                Ok(_) => {},
                Err(err) => {log::error!("DB Error: Failed update status for job id: {}\n Error output: {:?}", job_id, err)}   
            }

            if status == JobStatus::COMPLETED {
                metrics::JOBS_COMPLETED_TOTAL.inc();
            } else if status == JobStatus::FAILED {
                metrics::JOBS_FAILED_TOTAL.inc();
            }

            job.status = status;
        }
    }

    pub fn store_results(&mut self, job_id: Uuid, job_results: JobResult) {
        match db::insert_results(&self.connection, job_id, job_results.clone()) {
            Ok(_) => {},
            Err(err) => {log::error!("DB Error: Failed update insert results into the database. Job id: {}\n Error output: {:?}", job_id, err)}   
        }

        self.results.insert(job_id, job_results);
    }
}