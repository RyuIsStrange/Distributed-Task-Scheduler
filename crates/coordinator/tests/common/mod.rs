use actix_http::Request;
use actix_web::body::BoxBody;
use actix_web::dev::{Service, ServiceResponse};
use actix_web::{Error};
use actix_web::{App, test, web};
use rusqlite::Connection;
use tempfile::NamedTempFile;
use std::sync::Arc;
use tokio::sync::Mutex;


use coordinator::{api, db};
use coordinator::queue::JobQueue;

pub async fn setup_tests() -> (impl Service<Request, Response = ServiceResponse<BoxBody>, Error = Error>, NamedTempFile) {
    let tempfile = NamedTempFile::new().unwrap();
    let db_path = tempfile.path().to_string_lossy().into_owned();

    let db_connection = Connection::open(&db_path).unwrap();
    let _ = db::init(&db_connection);

    let db_close = db_connection.close();
    if db_close.is_err() {
        log::error!("The DB connection failed to close after initialization");
    }

    let queue = Arc::new(Mutex::new(JobQueue::new(&db_path)));  

    (test::init_service(
        App::new()
            .app_data(web::Data::new(queue.clone()))
            .service(
                web::scope("/metrics")
                    .route("", web::get().to(api::metrics))
            )
            .service(
                web::scope("/api")
                    .route("/health", web::get().to(api::health_check))
                    .route("/worker/register", web::post().to(api::register_worker))
                    .route("/worker/heartbeat", web::post().to(api::worker_heartbeat))
                    
                    .route("/job/next", web::get().to(api::next_job))
                    .route("/job/{job_id}/results", web::post().to(api::job_results))

                    .service(
                        web::scope("")
                            .route("/job", web::post().to(api::submit_job))
                            .route("/job/list", web::post().to(api::list_jobs))
                            .route("/job/{job_id}", web::get().to(api::job_details))
                    )
            )
    ).await,
    tempfile)
}