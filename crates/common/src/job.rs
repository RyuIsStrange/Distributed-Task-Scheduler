use serde::{
    Deserialize, 
    Serialize
};
use chrono::{
    DateTime, 
    Utc
};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Job {
    pub id: Uuid,
    pub command: String,
    pub args: Vec<String>, // Command Args
    pub status: JobStatus,
    pub timestamp: DateTime<Utc>,

    pub retry_count: u32,
    pub max_retries: u32,

    pub priority: Priority,

    pub schedule: Option<String>,
    pub next_run: Option<DateTime<Utc>>,
    pub is_recurring: Option<bool>,
    pub parent_schedule_id: Option<Uuid>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct JobResult {
    pub exitcode: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub enum JobStatus {
    PENDING,
    RUNNING,
    COMPLETED,
    FAILED,
    CANCELED,
    RETRYING
}

impl FromStr for JobStatus {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "PENDING" => Ok(JobStatus::PENDING),
            "RUNNING" => Ok(JobStatus::RUNNING),
            "COMPLETED" => Ok(JobStatus::COMPLETED),
            "FAILED" => Ok(JobStatus::FAILED),
            "CANCELED" => Ok(JobStatus::CANCELED),
            "RETRYING" => Ok(JobStatus::RETRYING),

            _ => Err(())
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Priority {
    HIGH = 0,
    MEDIUM = 1,
    LOW = 2
}

impl FromStr for Priority {
    type Err = ();

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        match input {
            "HIGH" => Ok(Priority::HIGH),
            "MEDIUM" => Ok(Priority::MEDIUM),
            "LOW" => Ok(Priority::LOW),

            _ => Err(())
        }
    }
}