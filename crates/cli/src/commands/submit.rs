use std::str::FromStr;
use uuid::Uuid;
use colored::Colorize;
use common::{job::{Job, Priority}, message::SubmitJobRequest};

use crate::client;


pub async fn job(command: String, args_str: Option<String>, priority: Option<String>, schedule: Option<String>, depends_on: Option<String>) {
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

    // TODO: Make error out gracefully without submitting job request

    let depend_ids: Option<Vec<Uuid>>;
    if let Some(deps) = depends_on {
        let v: Vec<String> = deps.split(",").map(String::from).collect();

        let mut collected_ids = vec![];

        for id in v {
            match Uuid::from_str(&id) {
                Ok(val) => { collected_ids.push(val); },
                Err(_) => {
                    println!("Invalid ID inputed: {}", id);
                    break;
                }
            }
        }

        depend_ids = Some(collected_ids)
    } else {
        depend_ids = None
    }

    let json = SubmitJobRequest {
        command: command,
        args: args,
        priority: p,
        schedule: schedule,
        depends_on: depend_ids
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