use actix_http::StatusCode;
use actix_web::test::{TestRequest, call_service, read_body_json};
use ::common::{job::Job, message::{SubmitJobListRequest, SubmitJobRequest}};

mod common;

// Health check, submit job, submit then status check, list jobs.
#[actix_web::test] 
async fn job_api() {
    let (app, _tempfile) = common::setup_tests().await;

    let health_req = call_service(
            &app, 
            TestRequest::with_uri("/api/health").to_request()
        ).await.status();
    assert_eq!(health_req, StatusCode::OK);

    let submit_job_req = call_service(
            &app, 
            TestRequest::post().uri("/api/job").set_json(
                SubmitJobRequest {
                    command: "whoami".to_string(),
                    args: vec![],
                    depends_on: None,
                    priority: None,
                    schedule: None
                }
            ).to_request()
        ).await;
    assert_eq!(submit_job_req.status(), StatusCode::OK);

    let submitted_job = read_body_json::<Job, _>(submit_job_req).await;
    let job_status_req = call_service(
            &app, 
            TestRequest::with_uri(&format!("/api/job/{}", submitted_job.id))
            .to_request()
        ).await;
    assert_eq!(job_status_req.status(), StatusCode::OK);
    
    let job_list_req = call_service(
            &app, 
            TestRequest::post().uri("/api/job/list")
            .set_json(SubmitJobListRequest { status_search: None })
            .to_request()
        ).await.status();
    assert_eq!(job_list_req, StatusCode::OK);
}