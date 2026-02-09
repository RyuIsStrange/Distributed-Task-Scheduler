use std::str::FromStr;
use colored::*;

use common::job::{JobStatus};

use crate::client;


pub async fn jobs(input: Option<String>) {
    let status_search;
    if input.is_some() {
        status_search = JobStatus::from_str(&input.as_ref().unwrap().to_uppercase())
    } else {
        status_search = Err("None")
    }

    if status_search.is_ok() || input.is_none() {
        let list = client::fetch_list(status_search).await;

        println!("{:#?}", list.unwrap().0)
    } else {
        println!("{}", "Invalid status search parameter".red());
    }
}