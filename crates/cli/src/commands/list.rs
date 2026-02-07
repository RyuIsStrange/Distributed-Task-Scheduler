use std::str::FromStr;
use colored::*;

use common::job::{JobStatus};


pub async fn jobs(input: Option<String>) {
    let status_search;
    if input.is_some() {
        status_search = JobStatus::from_str(&input.as_ref().unwrap())
    } else {
        status_search = Err("No")
    }

    if status_search.is_ok() || input == None {
        
    } else {
        println!("{}", "Invalid status search parameter".red());
    }
}