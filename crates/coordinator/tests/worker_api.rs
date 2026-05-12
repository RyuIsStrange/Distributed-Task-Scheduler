use actix_http::StatusCode;
use actix_web::test::{TestRequest, call_service};
use chrono::Utc;
use ::common::message::{WorkerHeartbeat, WorkerRegister};
use uuid::Uuid;

mod common;

// Worker registration, heartbeat, 404 on unregistered worker heartbeat.

#[actix_web::test]
async fn worker_api() {
    let (app, _tempfile) = common::setup_tests().await;
    
    let worker = WorkerRegister {
        worker_id: Uuid::new_v4(),
        hostname: hostname::get().unwrap_or_default().to_string_lossy().to_string(),
    };

    let unregistered_heartbeat = call_service(
            &app, 
            TestRequest::post()
            .uri("/api/worker/heartbeat")
            .set_json(
                WorkerHeartbeat {
                    worker_id: worker.worker_id,
                    timestamp: Utc::now()
                }
            )
            .to_request()
        ).await;
    assert_eq!(unregistered_heartbeat.status(), StatusCode::NOT_FOUND);

    let register = call_service(
            &app, 
            TestRequest::post()
            .uri("/api/worker/register")
            .set_json(&worker)
            .to_request()
        ).await;
    assert_eq!(register.status(), StatusCode::OK);

    let registered_heartbeat = call_service(
            &app, 
            TestRequest::post()
            .uri("/api/worker/heartbeat")
            .set_json(
                WorkerHeartbeat {
                    worker_id: worker.worker_id,
                    timestamp: Utc::now()
                }
            )
            .to_request()
        ).await;
    assert_eq!(registered_heartbeat.status(), StatusCode::OK);
}