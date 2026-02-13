use std::str::FromStr;

use common::{job::Priority, message::SubmitJobRequest};

use crate::client;


pub async fn job(command: String, args_str: Option<String>, priority: Option<String>, schedule: Option<String>) {
    let mut args = vec![];

    if args_str.is_some() {
        for arg in args_str.unwrap().split_ascii_whitespace() {
            args.push(arg.to_string());
        }
    }

    let json = SubmitJobRequest {
        command: command,
        args: args,
        priority: Some(Priority::from_str(&priority.unwrap_or("LOW".to_string())).unwrap()),
        schedule: schedule
    };

    let result = client::submit_job(json).await;

    println!("Result: {:#?}", result);
}