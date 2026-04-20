use common::{job::JobStatus, message::{ErrorMessage, GetJobListResponse, GetJobStatusResponse, SubmitJobListRequest, SubmitJobRequest}};
use reqwest::{Response, StatusCode};
use std::sync::LazyLock;

const TOO_MANY_REQUESTS: &str = "Slow down too many requests have been sent recently.";
const PARSE_ERROR_STRING: &str = "Unknown message from server.";
const FAILED_REQUEST_STRING: &str = "Failed to send request to server.";

static COORDINATOR_ADDR: LazyLock<String> = LazyLock::new(|| {
    let addr= std::env::var("COORDINATOR_ADDR");
    match addr {
        Ok(addr_string) => {
            addr_string
        },
        Err(_) => {
            println!("COORDINATOR_ADDR is not found. Defaulting to localhost:8080");
            String::from("127.0.0.1:8080")
        }
    }
});

pub async fn submit_job(submit_request: SubmitJobRequest) -> Result<Response, ErrorMessage> {
    let url = format!("http://{}/api/job", *COORDINATOR_ADDR);

    let client = reqwest::Client::new();

    match client.post(url).json(&submit_request).send().await {
        Ok(response) => {
            if response.status().is_success() {
                Ok(response)
            } else if response.status() == StatusCode::TOO_MANY_REQUESTS {
                Err(ErrorMessage::new(String::from("429"), TOO_MANY_REQUESTS.to_string()))
            } else {
                let error = response.json::<ErrorMessage>().await
                    .unwrap_or_else(|_| ErrorMessage::new(String::from("500"), PARSE_ERROR_STRING.to_string()));
                
                Err(error)
            }
        },
        Err(_) => {Err(ErrorMessage::new(String::from("503"), FAILED_REQUEST_STRING.to_string()))}
    }

}

pub async fn fetch_status(id: String) -> Result<GetJobStatusResponse, ErrorMessage> {
    let url = format!("http://{}/api/job/{}", *COORDINATOR_ADDR, id);

    match reqwest::get(&url).await {
        Ok(response) => {
            if response.status().is_success() {
                let json = response.json::<GetJobStatusResponse>().await
                    .map_err(|_| ErrorMessage::new(String::from("500"), PARSE_ERROR_STRING.to_string()))?;

                Ok(json)
            } else if response.status() == StatusCode::TOO_MANY_REQUESTS {
                Err(ErrorMessage::new(String::from("429"), TOO_MANY_REQUESTS.to_string()))
            } else {
                let error = response.json::<ErrorMessage>().await
                    .unwrap_or_else(|_| ErrorMessage::new(String::from("500"), PARSE_ERROR_STRING.to_string()));
                
                Err(error)
            }
        },
        Err(_) => {Err(ErrorMessage::new(String::from("503"), FAILED_REQUEST_STRING.to_string()))}
    }   
}

pub async fn fetch_list(status_search: Option<JobStatus>) -> Result<GetJobListResponse, ErrorMessage> {
    let url = format!("http://{}/api/job/list", *COORDINATOR_ADDR);

    let client = reqwest::Client::new();

    match client.post(&url).json(&SubmitJobListRequest { status_search }).send().await {
        Ok(response) => {
            if response.status().is_success() {
                let json = response.json::<GetJobListResponse>().await
                    .map_err(|_| ErrorMessage::new(String::from("500"), PARSE_ERROR_STRING.to_string()))?;

                Ok(json)
            } else if response.status() == StatusCode::TOO_MANY_REQUESTS {
                Err(ErrorMessage::new(String::from("429"), TOO_MANY_REQUESTS.to_string()))
            } else {
                let error = response.json::<ErrorMessage>().await
                    .unwrap_or_else(|_| ErrorMessage::new(String::from("500"), PARSE_ERROR_STRING.to_string()));
                
                Err(error)
            }
        },
        Err(_) => {Err(ErrorMessage::new(String::from("503"), FAILED_REQUEST_STRING.to_string()))}
    }

}