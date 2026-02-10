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

        if list.is_err() {
            println!("{}", "An error has occurred.".red());
            println!("Err printout: {:?}", list)
        } else if !list.as_ref().unwrap().list.clone().unwrap().is_empty() {
            println!("{:#?}", list.unwrap().list.unwrap())
        } else if list.as_ref().unwrap().list.clone().unwrap().is_empty() {
            println!("No jobs were found with search.")
        } else {
            println!("{}", "An error has occurred.".red());
            println!("Err printout: {:?}", list)
        }
    } else {
        println!("{}", "Invalid status search parameter".red());
    }
}