use clap::{Parser, Subcommand};

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
        #[arg()]
        command: String,
        #[arg(long)]
        args: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        schedule: Option<String>
    },
    
    /// Check job status
    Status {
        #[arg()]
        job_id: String,
    },
    
    /// List jobs
    List {
        #[arg(long)]
        status: Option<String>,
    },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Submit { command , args, priority, schedule} => { submit::job(command, args, priority, schedule).await; },
        Commands::Status { job_id } => { status::fetch(job_id).await; },
        Commands::List { status } => { list::jobs(status).await; }
    }
}