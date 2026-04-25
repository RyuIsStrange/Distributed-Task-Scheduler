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
        #[arg(help = "Command to run")]
        command: String,

        #[arg(long, help = "Arguments for the command")]
        args: Option<String>,

        #[arg(long, help = "Priority of the job. Options: High, Medium, or Low")]
        priority: Option<String>,
        
        #[arg(long, help = "5-6 Length cron schedule")]
        schedule: Option<String>,

        #[arg(long, value_delimiter(','), help = "UUID of job required to finish for this one to run\nExample: --depends-on UUID1, UUID2")]
        depends_on: Option<Vec<Uuid>>
    },
    
    /// Check job status
    Status {
        #[arg(help = "UUID of job to lookup")]
        job_id: String,
    },
    
    /// List jobs
    List {
        #[arg(long, help = "Status to filter jobs")]
        status: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    match cli.command {
        Commands::Submit { command , args, priority, schedule, depends_on} => { 
            submit::job(
                command, 
                args, 
                priority, 
                schedule,
                depends_on
            ).await;
        },

        Commands::Status { job_id } => { status::fetch(job_id).await; },

        Commands::List { status } => { list::jobs(status).await; }
    }
}