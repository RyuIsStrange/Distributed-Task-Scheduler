use common::job::{JobStatus, Priority};
use colored::*;

use crate::client;

pub async fn fetch(id: String) {
    let res = client::fetch_status(id.clone()).await;

    if res.is_ok() {
        let job_status_resp = res.unwrap();

        // I hate these one line string
        let print_response = format!(
            "Status for Job ID: {}\n\n{}\nPriority: {}\nRetry count: {}\n\nCommand: {}\nArguments: {:#?}\n\nTime Created (UTC): {}\n\n{}\n\n{}\n\n{}", 
            job_status_resp.job.id.to_string().blue(),

            if job_status_resp.job.status == JobStatus::CANCELED || job_status_resp.job.status == JobStatus::FAILED {
                format!("{}{}",
                    "Status: ".white(),
                    job_status_resp.job.status.to_string().red()
                )
            } else if job_status_resp.job.status == JobStatus::COMPLETED {
                format!("{}{}",
                    "Status: ".white(),
                    job_status_resp.job.status.to_string().green()
                )
            } else {
                format!("{}{}",
                    "Status: ".white(),
                    job_status_resp.job.status.to_string().yellow()
                )
            },
            if job_status_resp.job.priority == Priority::HIGH{
                job_status_resp.job.priority.to_string().red()
            } else if job_status_resp.job.priority == Priority::MEDIUM {
                job_status_resp.job.priority.to_string().yellow()
            } else {
                job_status_resp.job.priority.to_string().green()
            },
            if job_status_resp.job.max_retries == job_status_resp.job.retry_count {
                job_status_resp.job.retry_count.to_string().red()
            } else {
                job_status_resp.job.retry_count.to_string().green()
            },

            job_status_resp.job.command.blue(),
            job_status_resp.job.args,

            job_status_resp.job.timestamp.to_utc().to_string().blue(),

            if job_status_resp.result.is_some() {
                format!("Results: \n\tExit Code: {} \n\tOutput: {} \n\tError: {}", 
                    if job_status_resp.result.clone().unwrap().exitcode == 0 {
                        job_status_resp.result.clone().unwrap().exitcode.to_string().green()
                    } else {
                        job_status_resp.result.clone().unwrap().exitcode.to_string().red()
                    }, 
                    job_status_resp.result.clone().unwrap().stdout.white(), 
                    job_status_resp.result.unwrap().stderr.red()
                )
            } else if job_status_resp.job.is_recurring.unwrap() {
                format!("{}",
                    "Schedule jobs wont have results, find one if its spawned jobs.".blue()
                )
            } else {
                format!("{}",
                    "No results yet".yellow()
                )
            },

            if job_status_resp.job.parent_schedule_id.is_some() {
                job_status_resp.job.parent_schedule_id.unwrap().to_string().green()
            } else if job_status_resp.job.is_recurring.unwrap() { 
                "Scheduled job".to_string().blue()
            }else {
                "This job has no scheduled job parent".to_string().white()
            },

            if job_status_resp.job.is_recurring.unwrap() {
                format!("Schedule Info: \n\tSchedule: {} \n\tRecurring: {} \n\tNext run time {}", 
                    job_status_resp.job.schedule.unwrap(), 
                    job_status_resp.job.is_recurring.unwrap(), 
                    job_status_resp.job.next_run.unwrap()
                ).green()
            } else {
                "This job isn't a scheduled job".to_string().white()
            }
        );

        println!("{}", print_response)
    } else {
        println!("{}", "There was an error when fetching job.".red());
        // if res.is_ok() && res.as_ref().unwrap().1 == StatusCode::from_u16(400).unwrap() {
        //     println!("{} {}", "Status code: 400. Failed to parse UUID: ".red(), id);
        // } else if res.is_ok() && res.as_ref().unwrap().1 == StatusCode::from_u16(404).unwrap() {
        //     println!("{} {}", "Status code: 404. Failed to find job with UUID: ".red(), id);
        // } else {
        //     println!("Err: {:#?}", res.err());
        // }
    }
}