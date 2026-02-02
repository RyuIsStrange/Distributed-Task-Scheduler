use common::{
    message::{
        WorkerHeartbeat, 
        NextJobRequest, 
        WorkerRegister
    },
    job::JobResult
};
use reqwest::{
    Response,
    Error,
};
use chrono::Utc;
use uuid::Uuid;

const COORDINATOR_ADDR: &str = "127.0.0.1:8080";

pub async fn register_worker(worker: WorkerRegister) {
    let url = format!("http://{}/api/worker/register", COORDINATOR_ADDR);

    let client = reqwest::Client::new();

    let _ = client.post(url)
        .header("Content-Type", "application/json")
        .json(&worker)
        .send()
        .await;
}

pub async fn send_heartbeat(worker_id: Uuid) {
    let url = format!("http://{}/api/worker/heartbeat", COORDINATOR_ADDR);

    let heartbeat = WorkerHeartbeat {
        worker_id,
        timestamp: Utc::now()
    };

    let client = reqwest::Client::new();

    let _ = client.post(url)
        .header("Content-Type", "application/json")
        .json(&heartbeat)
        .send()
        .await;
}

pub async fn get_next_job(worker_id: Uuid) -> Result<Response, Error> {
    let url = format!("http://{}/api/job/next", COORDINATOR_ADDR);

    let client = reqwest::Client::new();

    let jobreq = NextJobRequest { worker_id };

    let response = client.get(url)
        .header("Content-Type", "application/json")
        .json(&jobreq)
        .send()
        .await;

    response
}

pub async fn post_job_results(results: JobResult, job_id: Uuid) {
    let url = format!("http://{}/api/job/{}/results", COORDINATOR_ADDR, job_id);

    let client = reqwest::Client::new();

    let _ = client.post(url)
        .header("Content-Type", "application/json")
        .json(&results)
        .send()
        .await;
}