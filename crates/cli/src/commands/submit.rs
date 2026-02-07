use std::str::FromStr;

use common::{job::Priority};
use serde_json::json;

use crate::client;


pub async fn job(command: String, args_str: Option<String>, priority: Option<String>, schedule: Option<String>) {
    let mut args = vec![];

    if args_str.is_some() {
        if args_str.iter().len() != 0 {
            for arg in args_str.unwrap().split_ascii_whitespace() {
                args.push(arg.to_string());
            }
        }
    }

    let json = json!({
        "command": command,
        "args": args,
        "priority": Priority::from_str(&priority.unwrap_or("LOW".to_string())).unwrap(),
        "schedule": schedule.or_else(|| Some("".to_string()))
    });

    let result = client::submit_job(json).await;

    println!("Result: {:#?}", result);
}