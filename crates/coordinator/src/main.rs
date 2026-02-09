use actix_web::{
    middleware::Logger, 
    HttpServer, 
    App, 
    web
};
use std::{
    time::Duration,
    io::Result, 
    sync::Arc,
};
use tokio::{
    sync::Mutex, 
    time::sleep
};
use rusqlite::Connection;

use crate::queue::JobQueue;

mod api; mod queue; mod db;

#[actix_web::main]
async fn main() -> Result<()> {
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));

    log::info!("Initalizing database..");
    let db = Connection::open("scheduler.db").unwrap();
    let _ = db::init(&db);

    let queue = Arc::new(Mutex::new(JobQueue::new()));
    
    let checker_queue = queue.clone();
    tokio::spawn(async move {
        loop {
            let q = checker_queue.clone();

            log::info!("Checking workers...");

            api::check_workers(q).await;
            sleep(Duration::from_secs(30)).await;    
        }
    });

    let schedule_queue = queue.clone();
    tokio::spawn(async move {
        loop {
            let q = schedule_queue.clone();
            
            log::info!("Check scheduled jobs");

            api::check_schedules(q).await;
            sleep(Duration::from_secs(60)).await;
        }
    });

    log::info!("Starting api server...");

    HttpServer::new(move || {
        App::new()
            .wrap(Logger::default())
            .app_data(web::Data::new(queue.clone()))
            .service(
                web::scope("/api")
                    .route("/health", web::get().to(api::health_check))
                    .route("/worker/register", web::post().to(api::register_worker))
                    .route("/worker/heartbeat", web::post().to(api::worker_heartbeat))
                    .route("/job", web::post().to(api::submit_job))
                    .route("/job/list", web::get().to(api::list_jobs))
                    .route("/job/next", web::get().to(api::next_job))
                    .route("/job/{job_id}", web::get().to(api::job_details))
                    .route("/job/{job_id}/results", web::post().to(api::job_results))
            )
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}