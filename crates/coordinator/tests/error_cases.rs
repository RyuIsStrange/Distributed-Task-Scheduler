use actix_http::StatusCode;
use actix_web::test::{TestRequest, call_service};
use ::common::message::SubmitJobRequest;
use uuid::Uuid;

mod common;

// 404 on unknown job, 400 on bad UUID, 400 on invalid dep UUID.
#[actix_web::test]
async fn api_errors() {
    let (app, _tempfile) = common::setup_tests().await;
    
    let unkown_job = call_service(
            &app, 
            TestRequest::with_uri(&format!("/api/job/{}", Uuid::new_v4()))
            .to_request()
        ).await;
    assert_eq!(unkown_job.status(), StatusCode::NOT_FOUND);

    let bad_uuid = call_service(
            &app, 
            TestRequest::with_uri("/api/job/not_a_real_uuid")
            .to_request()
        ).await;
    assert_eq!(bad_uuid.status(), StatusCode::BAD_REQUEST);

    let invalid_dependency = call_service(
            &app, 
            TestRequest::post().uri("/api/job")
            .set_json(
                SubmitJobRequest {
                    command: "whoami".to_string(),
                    args: vec![],
                    depends_on: Some(vec![Uuid::new_v4()]),

                    priority: None,
                    schedule: None
                }
            )
            .to_request()
        ).await;
    assert_eq!(invalid_dependency.status(), StatusCode::BAD_REQUEST);
}