use common::{
    message::{
        WorkerHeartbeat, 
        NextJobRequest, 
        WorkerRegister
    },
    job::JobResult
};
use reqwest::{
    Error, Response, StatusCode
};
use std::{sync::LazyLock, time::Duration};
use tokio::time::sleep;
use chrono::Utc;
use uuid::Uuid;

const MAX_RETRIES: i32 = 3;

static COORDINATOR_ADDR: LazyLock<String> = LazyLock::new(|| {
    let addr= std::env::var("COORDINATOR_ADDR");
    match addr {
        Ok(addr_string) => {
            addr_string
        },
        Err(_) => {
            log::info!("COORDINATOR_ADDR is not found. Defaulting to localhost:8080");
            String::from("127.0.0.1:8080")
        }
    }
});

// We let loop forever as it work do work until it connects/registers
pub async fn register_worker(worker: WorkerRegister) {
    loop {
        let url = format!("http://{}/api/worker/register", *COORDINATOR_ADDR);

        let client = reqwest::Client::new();

        match client.post(url)
            .header("Content-Type", "application/json")
            .json(&worker)
            .send()
            .await 
            {
            Ok(_) => {break;},
            Err(err) => {
                log::error!("Failed to register with Coordinator. Retrying in 10 seconds!");
                log::error!("Connection error: {}", err);
                sleep(Duration::from_secs(10)).await;
            }
        }
    }
}

pub async fn send_heartbeat(worker_id: Uuid, worker: &WorkerRegister) {
    let url = format!("http://{}/api/worker/heartbeat", *COORDINATOR_ADDR);

    let heartbeat = WorkerHeartbeat {
        worker_id,
        timestamp: Utc::now()
    };

    let client = reqwest::Client::new();

    match client.post(url)
        .header("Content-Type", "application/json")
        .json(&heartbeat)
        .send()
        .await 
        {
        Ok(response) => {
            if response.status() == StatusCode::NOT_FOUND {
                log::info!("Detected that worker isn't connected to coordinator. Re-regestering.");
                register_worker(worker.clone()).await;
            }
        },
        Err(_) => {
            log::error!("Failed to send heartbeat to coordinator")
        }
    }
}

pub async fn get_next_job(worker_id: Uuid) -> Result<Response, Error> {
    let url = format!("http://{}/api/job/next", *COORDINATOR_ADDR);

    let client = reqwest::Client::new();

    let jobreq = NextJobRequest { worker_id };

    let response = client.get(url)
        .header("Content-Type", "application/json")
        .json(&jobreq)
        .send()
        .await;

    response
}

// Only loop 3 times (Or what MAX_RETRIES is set to) then we can assume coordinator is offline or networking error and log the job_id w/result
pub async fn post_job_results(results: JobResult, job_id: Uuid) {
    for i in 1..=MAX_RETRIES {
        let url = format!("http://{}/api/job/{}/results", *COORDINATOR_ADDR, job_id);

        let client = reqwest::Client::new();

        match client.post(url)
            .header("Content-Type", "application/json")
            .json(&results)
            .send()
            .await 
            {
            Ok(_) => {break;/* Break out of the loop as results got submited */},
            Err(_) => {
                if i != MAX_RETRIES {
                    log::error!("Failed to submit results for job with ID: {}. Retrying after 10 seconds!", job_id);
                    sleep(Duration::from_secs(10)).await;
                } else {
                    log::error!("Failed to submit results for job after {} retries.\nJob: {}\nResults: {:#?}", MAX_RETRIES, job_id, results)
                }
            }
        }
    }
}