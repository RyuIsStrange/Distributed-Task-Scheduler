use common::job::{
    JobResult, 
    Job
};
use tokio::process::Command;

const FAILED_MESSAGE: &str = "The command has failed. Check permission or if command exist.";

pub async fn execute(job: Job) -> JobResult {
    #[cfg(target_os = "windows")] {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", &job.command]);

        cmd.args(&job.args);
        let output = cmd.output().await;

        match output {
            Ok(o) => {
                return JobResult { 
                    exitcode: o.status.code().unwrap_or(-1), 
                    stdout: String::from_utf8_lossy(&o.stdout).to_string(), 
                    stderr: String::from_utf8_lossy(&o.stderr).to_string()
                }
            },
            Err(_) => {
                return JobResult {
                    exitcode: 1, 
                    stdout: String::from(""), 
                    stderr: String::from(FAILED_MESSAGE)
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))] {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", &job.command]);

        cmd.args(&job.args);
        let output = cmd.output().await;

        match output {
            Ok(o) => {
                return JobResult { 
                    exitcode: o.status.code().unwrap_or(-1), 
                    stdout: String::from_utf8_lossy(&o.stdout).to_string(), 
                    stderr: String::from_utf8_lossy(&o.stderr).to_string()
                }
            },
            Err(_) => {
                return JobResult {
                    exitcode: 1, 
                    stdout: String::from(""), 
                    stderr: String::from(FAILED_MESSAGE)
                }
            }
        }
    }
    
}