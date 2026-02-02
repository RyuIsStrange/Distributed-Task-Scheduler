use common::{
    message::WorkerRegister,
    job::Job, 
};
use tokio::time::{
    sleep,
    self, 
};
use std::time::Duration;
use uuid::Uuid;

use crate::executor::execute;

mod client; mod executor;

const HEARTBEAT_INTERVAL: u64 = 10;

#[tokio::main]
async fn main() {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    let hostname = hostname::get().unwrap_or_default().to_string_lossy().to_string();
    let worker_id = Uuid::new_v4();

    let worker = WorkerRegister {worker_id, hostname: hostname.clone()};

    client::register_worker(worker).await;

    log::info!("Registered with coordinator with ID {} and hostname {}", worker_id, hostname);

    tokio::spawn(async move {
        loop {
            client::send_heartbeat(worker_id).await;
            sleep(Duration::from_secs(HEARTBEAT_INTERVAL)).await;
        }
    });

    loop {
        match client::get_next_job(worker_id.clone()).await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<Job>().await {
                        Ok(job) => {
                            log::info!("Got job: {:?}", job);
                            let results = execute(job.clone()).await;
                            log::info!("Sending result to coordinator");
                            client::post_job_results(results, job.id).await;
                        }
                        Err(e) => log::error!("Failed to parse job: {}", e),
                    }
                } else {
                    time::sleep(Duration::from_secs(5)).await;
                }
            },
            Err(e) => {
                log::error!("Request failed, likely 404 (no jobs): {}", e);
                time::sleep(Duration::from_secs(5)).await;
            }
        }    
    }
}