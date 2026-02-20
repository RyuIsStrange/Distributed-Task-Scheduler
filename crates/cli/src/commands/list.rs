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
        } else {
            let list_unwrap = list.unwrap().list.unwrap();
            
            if !list_unwrap.is_empty() {
                println!("{:#?}", list_unwrap)
            } else {
                println!("No jobs were found with search.")
            }
        }
    } else {
        println!("{}", "Invalid status search parameter".red());
    }
}