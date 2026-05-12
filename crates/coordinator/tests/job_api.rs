use actix_http::StatusCode;
use actix_web::test::{TestRequest, call_service};
use ::common::message::SubmitJobRequest;

mod common;

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
            TestRequest::with_uri("/api/job").app_data(
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

    let job_status_req = call_service(
            &app, 
            TestRequest::with_uri(&format!("/api/job/{}", todo!("Get the job UUID from the submitted job then run the status request with that UUID")))
            .to_request()
        ).await;
    // assert_eq!(job_status_req.status(), StatusCode::OK);
    
    let job_list_req = call_service(
            &app, 
            TestRequest::with_uri("/api/job/list")
            .to_request()
        ).await.status();
    assert_eq!(job_list_req, StatusCode::OK);
}