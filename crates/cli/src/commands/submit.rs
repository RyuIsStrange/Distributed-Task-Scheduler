use std::str::FromStr;
use uuid::Uuid;
use colored::Colorize;
use common::{job::{Job, Priority}, message::SubmitJobRequest};

use crate::client;


pub async fn job(command: String, args_str: Option<String>, priority: Option<String>, schedule: Option<String>, depends_on: Option<Uuid>) {
    let mut args = vec![];

    if args_str.is_some() {
        for arg in args_str.unwrap().split_ascii_whitespace() {
            args.push(arg.to_string());
        }
    }

    let unwrap_priority;
    let p: Option<Priority>;
    if priority.is_some() {
        unwrap_priority = priority.unwrap().to_uppercase();

        if Priority::from_str(&unwrap_priority).is_err() {
            println!("Invalid priority value, must be one of the following: Low, Medium, High");
            return;
        } else {
            p = Priority::from_str(&unwrap_priority).ok();
        }
    } else {
        p = Some(Priority::LOW);
    }

    let json = SubmitJobRequest {
        command: command,
        args: args,
        priority: p,
        schedule: schedule,
        dependent: depends_on
    };

    let result = client::submit_job(json).await;

    if let Ok(r) = result {
        match r.json::<Job>().await {
            Ok(json) => println!("Job submited with ID: {}", json.id),
            Err(_) => println!("{} Job was submited but failed to generate JSON response.", "Err:".red())
        }
    } else {
        println!("{} Job failed to be submited.", "Err:".red());
    }
}