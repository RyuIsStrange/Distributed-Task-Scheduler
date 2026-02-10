use common::{job::JobStatus, message::{GetJobListResponse, GetJobStatusResponse, SubmitJobListRequest, SubmitJobRequest}};
use reqwest::{Error, Response, StatusCode};

const COORDINATOR_ADDR: &str = "127.0.0.1:8080";

pub async fn submit_job(submit_request: SubmitJobRequest) -> Result<Response, Error> {
    let url = format!("http://{}/api/job", COORDINATOR_ADDR);

    let client = reqwest::Client::new();

    let post = client.post(url).json(&submit_request).send().await;

    post
}

pub async fn fetch_status(id: String) -> Result<(GetJobStatusResponse, StatusCode), Error> {
    let url = format!("http://{}/api/job/{}", COORDINATOR_ADDR, id);

    let response = reqwest::get(&url).await?;

    let status = response.status();

    let json = response.json::<GetJobStatusResponse>().await?;

    Ok((json, status))
}

pub async fn fetch_list(status_search: Result<JobStatus, &str>) -> Result<GetJobListResponse, Error> {
    let url = format!("http://{}/api/job/list", COORDINATOR_ADDR);

    let client = reqwest::Client::new();

    let response;
    if status_search.is_ok() {
        response = client.post(&url).json(&SubmitJobListRequest { status_search: Some(status_search.unwrap())}).send().await?;
    } else {
        response = client.post(&url).json(&SubmitJobListRequest { status_search: None}).send().await?;
    }

    let json = response.json::<GetJobListResponse>().await?;

    Ok(json)
}