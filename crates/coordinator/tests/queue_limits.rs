use actix_http::StatusCode;
use actix_web::test::{TestRequest, call_service};
use ::common::message::SubmitJobRequest;

mod common;

// Queue max size rejection
// We put this test by its self over putting in error_cases because we want to change the queue size env var without effecting other test

#[actix_web::test]
async fn queue_limit() {
    unsafe { std::env::set_var("MAX_QUEUE_SIZE", "2") };

    let (app, _tempfile) = common::setup_tests().await;

    for _ in 0..2 {
        call_service(
            &app, 
            TestRequest::post().uri("/api/job")
            .set_json(
                SubmitJobRequest {
                    command: "whoami".to_string(),
                    args: vec![],
                    depends_on: None,

                    priority: None,
                    schedule: None
                }
            )
            .to_request()
        ).await;
    }

    let rate_limited_job = call_service(
            &app, 
            TestRequest::post().uri("/api/job")
            .set_json(
                SubmitJobRequest {
                    command: "whoami".to_string(),
                    args: vec![],
                    depends_on: None,

                    priority: None,
                    schedule: None
                }
            )
            .to_request()
        ).await;
    assert_eq!(rate_limited_job.status(), StatusCode::BAD_REQUEST);
}