use clap::{Parser, Subcommand};
use uuid::Uuid;

use crate::commands::{list, status, submit};

mod commands; mod client;

#[derive(Parser)]
#[command(name = "scheduler")]
#[command(about = "Distributed task scheduler CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Submit new job
    Submit {
        /// Command to run
        #[arg()]
        command: String,
        /// Arguments for the command
        #[arg(long)]
        args: Option<String>,
        /// Priority of the job. Options: High, Medium, or Low
        #[arg(long)]
        priority: Option<String>,
        /// 5-6 Length cron schedule
        #[arg(long)]
        schedule: Option<String>,
        /// UUID of job required to finish for this one to run
        #[arg(long)]
        dependant: Option<Uuid>
    },
    
    /// Check job status
    Status {
        #[arg()]
        job_id: String,
    },
    
    /// List jobs
    List {
        /// Status to filter jobs by
        #[arg(long)]
        status: Option<String>,
    },
}

// TODO: Make error prompting in CLI more then "Error has occurred" and to parse the error and inform the user what caused it.

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Submit { command , args, priority, schedule, dependant} => { 
            submit::job(
                command, 
                args, 
                priority, 
                schedule,
                dependant
            ).await; 
        },

        Commands::Status { job_id } => { status::fetch(job_id).await; },

        Commands::List { status } => { list::jobs(status).await; }
    }
}