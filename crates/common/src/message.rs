use serde::{
    Deserialize, 
    Serialize
};
use chrono::{
    DateTime, 
    Utc
};
use uuid::Uuid;

use crate::job::{
    Job, JobResult, JobStatus, Priority 
};

// Client -> Coord 

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubmitJobRequest {
    pub command: String,
    pub args: Vec<String>,
    
    pub priority: Option<Priority>,

    pub schedule: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubmitJobResponse {
    pub job: Job,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetJobStatusResponse {
    pub job: Job,
    pub result: Option<JobResult>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SubmitJobListRequest {
    pub status_search: Option<JobStatus>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct GetJobListResponse {
    pub list: Option<Vec<Job>>,
}

// Worker -> Coord

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkerInfo {
    pub worker_id: Uuid,
    pub hostname: String,
    pub last_seen: DateTime<Utc>,
    pub status: WorkerStatus,
    pub current_job_id: Option<Uuid>
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum WorkerStatus {
    ALIVE,
    DEAD
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkerRegister {
    pub worker_id: Uuid,
    pub hostname: String
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct WorkerHeartbeat {
    pub worker_id: Uuid,
    pub timestamp: DateTime<Utc>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NextJobRequest {
    pub worker_id: Uuid
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobResultReport {
    pub job_id: Uuid,
    pub worker_id: Uuid,
    pub job_result: JobResult,
    pub finished_at: DateTime<Utc>
}

// Coord -> Worker

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct NextJobResponse {
    pub job: Option<Job>
}

// Error

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ErrorMessage {
    pub code: String,
    pub message: String
}