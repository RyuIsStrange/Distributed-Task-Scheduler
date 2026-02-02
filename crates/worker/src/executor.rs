use common::job::{
    JobResult, 
    Job
};
use tokio::process::Command;

pub async fn execute(job: Job) -> JobResult {
    #[cfg(target_os = "windows")] {
        let mut cmd = Command::new("cmd");
        cmd.args(["/C", &job.command]);

        cmd.args(&job.args);
        let output = cmd.output().await;

        let out_unwrap = output.unwrap();

        JobResult { 
            exitcode: out_unwrap.status.code().unwrap_or(-1), 
            stdout: String::from_utf8_lossy(&out_unwrap.stdout).to_string(), 
            stderr: String::from_utf8_lossy(&out_unwrap.stderr).to_string()
        }
    }

    #[cfg(not(target_os = "windows"))] {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", &job.command]);

        cmd.args(&job.args);
        let output = cmd.output().await;

        let out_unwrap = output.unwrap();

        JobResult { 
            exitcode: out_unwrap.status.code().unwrap_or(-1), 
            stdout: String::from_utf8_lossy(&out_unwrap.stdout).to_string(), 
            stderr: String::from_utf8_lossy(&out_unwrap.stderr).to_string()
        }
    }
    
}