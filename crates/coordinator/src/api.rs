use common::{
    job::{
        Job, JobResult, JobStatus, Priority 
    }, 
    message::{
        GetJobListResponse, NextJobRequest, SubmitJobListRequest, SubmitJobRequest, WorkerHeartbeat, WorkerInfo, WorkerRegister, WorkerStatus 
    }
};
use actix_web::{
    HttpResponse,
    Responder, 
    web
};
use cron::Schedule;
use tokio::sync::Mutex;
use std::{str::FromStr, sync::Arc};
use chrono::Utc;
use uuid::Uuid;

use crate::queue::JobQueue;

const MAX_RETRYS: u32 = 3;

// Health

pub async fn health_check() -> impl Responder {
    HttpResponse::Ok().json({
        Some(serde_json::json!({
            "status": "ok",
            "timestamp": Utc::now().to_rfc3339()
        }))
    })
}

// Worker

pub async fn check_workers(queue: Arc<Mutex<JobQueue>>) {
    let mut q = queue.lock().await;

    JobQueue::check_worker(&mut q);
}

pub async fn register_worker(
    req: web::Json<WorkerRegister>,
    queue: web::Data<Arc<Mutex<JobQueue>>>
) -> impl Responder {
    let mut q = queue.lock().await;

    let worker = WorkerInfo {
        worker_id: req.worker_id,
        hostname: req.hostname.clone(),
        last_seen: Utc::now(),
        status: WorkerStatus::ALIVE,
        current_job_id: None
    };

    log::info!("New worker regestered. Hostname: {} and ID: {}", req.worker_id.clone(), req.hostname.clone());

    JobQueue::register_worker(&mut q, worker.clone());
    HttpResponse::Ok().json(worker)
}

pub async fn worker_heartbeat(
    req: web::Json<WorkerHeartbeat>,
    queue: web::Data<Arc<Mutex<JobQueue>>>
) -> impl Responder {
    let mut q = queue.lock().await;

    JobQueue::update_worker_heartbeat(&mut q, req.clone());
    HttpResponse::Ok().finish()
}

pub async fn next_job(
    req: web::Json<NextJobRequest>,
    queue: web::Data<Arc<Mutex<JobQueue>>>
) -> impl Responder {
    let mut q = queue.lock().await;

    log::info!("A worker has polled a new job.");

    match JobQueue::get_next_job(&mut q, req.worker_id) {
        Some(result) => HttpResponse::Ok().json(result),
        None => HttpResponse::NotFound().body("No job in queue")
    }
}

// Job

pub async fn submit_job(
    req: web::Json<SubmitJobRequest>,
    queue: web::Data<Arc<Mutex<JobQueue>>>
) -> impl Responder {
    let mut q = queue.lock().await;

    #[allow(irrefutable_let_patterns)]
    let (schedule, next_run, is_recurring) = if req.schedule.is_some() {
        let valid_sched = Schedule::from_str(&req.schedule.clone().unwrap());
        
        let five_sched = if req.schedule.as_ref().unwrap().split_whitespace().count() == 5 {
            Schedule::from_str(&format!("0 {}", &req.schedule.clone().unwrap())).is_ok()
        } else { false };

        if let sched = req.schedule.as_ref().unwrap() && (valid_sched.is_ok() || five_sched) {
            let cron_expr = if five_sched {
                format!("0 {}", sched)
            } else {
                sched.to_string()
            };

            let next = Schedule::from_str(&cron_expr)
                .ok()
                .and_then(|s| s.upcoming(Utc).next());
        
            (Some(cron_expr), next, Some(true))
        }
        else {
            (None, None, Some(false))
        }
    } else {
        (None, None, Some(false))
    };

    let job = Job {
        id: Uuid::new_v4(),
        command: req.command.clone(),
        args: req.args.clone(),
        status: JobStatus::PENDING,
        timestamp: Utc::now(),
        
        retry_count: 0,
        max_retries: MAX_RETRYS,

        priority: req.priority.clone().unwrap_or(Priority::LOW),

        schedule,
        next_run,
        is_recurring,
        parent_schedule_id: None
    };


    if is_recurring == Some(true) {
        log::info!("New scheduled job added. Job info: id: {:?}, cmd: {:?}, args: {:?}", job.id, job.command, job.args);
        JobQueue::add_scheduled_jobs(&mut q, job.clone());
    } else {
        log::info!("New job added. Job info: id: {:?}, cmd: {:?}, args: {:?}", job.id, job.command, job.args);
        JobQueue::add_job(&mut q, job.clone());
    }
    HttpResponse::Ok().json( job )
}

pub async fn check_schedules(queue: Arc<Mutex<JobQueue>>) {
    let mut q = queue.lock().await;

    JobQueue::check_scheduled_jobs(&mut q);
}

// Results

pub async fn job_results(
    req: web::Json<JobResult>,
    path: web::Path<String>,
    queue: web::Data<Arc<Mutex<JobQueue>>>
) -> impl Responder {
    let job_id = Uuid::parse_str(&path.into_inner()).expect("Err");
    let mut q = queue.lock().await;
    let job = JobQueue::get_job(&mut q, job_id.clone());

    let results = JobResult {
        exitcode: req.exitcode,
        stdout: req.stdout.clone(),
        stderr: req.stderr.clone()
    };

    // Debug
    log::info!("A new result has been submitted Job ID: {}, Results: {:?}", job_id.clone(), results.clone());

    if results.exitcode != 0 && job.is_some() {
        let j = job.unwrap();
        if j.retry_count < j.max_retries {
            JobQueue::retry_job(&mut q, job_id);
            log::error!("Job ID: {} has failed and is being retried.", job_id);
        } else {
            JobQueue::store_results(&mut q, job_id, results.clone());
            JobQueue::update_job_status(&mut q, job_id, JobStatus::FAILED);
            log::error!("Job ID: {} has failed after max retries.", job_id);
        }
    } else {
        JobQueue::store_results(&mut q, job_id, results.clone());
        JobQueue::update_job_status(&mut q, job_id, JobStatus::COMPLETED);
    }

    HttpResponse::Ok().json(results)
}

pub async fn job_details(
    path: web::Path<String>,
    queue: web::Data<Arc<Mutex<JobQueue>>>
) -> impl Responder {
    let mut q = queue.lock().await;

    let id = path.into_inner();

    if Uuid::parse_str(&id).is_ok() {
        let job_id = Uuid::parse_str(&id).unwrap();
        let details = JobQueue::get_job_status(&mut q, job_id);

        if details.is_some() {
            HttpResponse::Ok().json(details)
        } else {
            HttpResponse::NotFound().finish() // No job? Return 404
        }
    } else {
        HttpResponse::BadRequest().finish() // Can't parse Uuid/not valid Uuid? Return 400
    }
}

pub async fn list_jobs(
    req: web::Json<SubmitJobListRequest>,
    queue: web::Data<Arc<Mutex<JobQueue>>>
) -> impl Responder {
    let q = queue.lock().await;
    
    let response = JobQueue::get_list(&q, req.status_search.clone());

    if response.is_ok() {
        HttpResponse::Ok().json(GetJobListResponse{ list: Some(response.unwrap())})
    } else {
        HttpResponse::BadRequest().finish()
    }
}