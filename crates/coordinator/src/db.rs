use common::job::{
        JobResult, 
        JobStatus, 
        Priority,
        Job,
};
use rusqlite::{
    Connection, Error, params, types::Type
};
use std::str::FromStr;
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub fn init(conn: &Connection) -> Result<(), Error> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS jobs (
            id UUID PRIMARY Key,
            command TEXT,
            args TEXT,
            status TEXT,
            timestamp TIMESTAMP,
            
            retry_count INTEGER,
            max_retries INTEGER,
            
            priority TEXT,

            schedule TEXT,
            next_run TIMESTAMP,
            is_recurring BOOL,
            parent_schedule_id UUID,

            depends_on TEXT
        );",
        ()
    )?;

    conn.execute(
            "CREATE TABLE IF NOT EXISTS results (
            id UUID,
            exitcode INTEGER,
            stdout TEXT,
            stderr TEXT,
            FOREIGN KEY(id) REFERENCES jobs(id)
        );",
        ()
    )?;

    Ok(())
}

pub fn insert_job(conn: &Connection, job: Job) -> Result<(), Error> {
    conn.execute(
        "INSERT INTO jobs (id, command, args, status, timestamp, retry_count, max_retries, priority, schedule, next_run, is_recurring, parent_schedule_id, depends_on) 
        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)", 
        (
            job.id.to_string(), 
            job.command, 
            serde_json::to_string(&job.args).map_err(|err| Error::ToSqlConversionFailure(Box::new(err)))?, 
            job.status.to_string(), 
            job.timestamp.to_rfc3339(), 

            job.retry_count,
            job.max_retries,

            job.priority.to_string(),

            job.schedule.unwrap_or_else(|| "None".to_string()),
            job.next_run.unwrap_or_else(|| Utc::now()).to_rfc3339(),
            job.is_recurring,
            job.parent_schedule_id.map(|id| id.to_string()),

            serde_json::to_string(&job.depends_on).map_err(|err| Error::ToSqlConversionFailure(Box::new(err)))?
        ),
    )?;

    Ok(())
}

pub fn insert_results(conn: &Connection, job_id: Uuid, results: JobResult) -> Result<(), Error> {
    conn.execute(
        "INSERT INTO results (id, exitcode, stdout, stderr) VALUES (?1, ?2, ?3, ?4)", 
        (job_id.to_string(), results.exitcode, results.stdout, results.stderr),
    )?;

    Ok(())
}

pub fn update_schedule_run(conn: &Connection, id: Uuid, next_run: DateTime<Utc>) -> Result<(), Error> {
    conn.execute(
        "UPDATE jobs SET next_run = ?1 WHERE id = ?2", 
        (next_run.to_rfc3339(), id.to_string())
    )?;

    Ok(())
}

pub fn update_job_status(conn: &Connection, job_id: Uuid, status: JobStatus) -> Result<(), Error> {
    conn.execute(
        "UPDATE jobs SET status = ?1 WHERE id = ?2",
        (format!("{:?}", status), job_id.to_string()),
    )?;

    Ok(())
}

pub fn update_retry_count(conn: &Connection, job_id: Uuid, count: u32) -> Result<(), Error> {
    conn.execute(
        "UPDATE jobs SET retry_count = ?1 WHERE id = ?2", 
        (count, job_id.to_string()),
    )?;


    Ok(())
}

pub fn fetch_from_db(conn: &Connection, status: Option<JobStatus>) -> Result<Vec<Job>, Error> {
    let (mut stmt, param) = if let Some(s) = status {
        (conn.prepare(
            "SELECT id, command, args, status, timestamp, retry_count, max_retries, priority, schedule, is_recurring, next_run, parent_schedule_id, depends_on
            FROM jobs 
            WHERE status IN (?1)
            ORDER BY timestamp ASC"
        )?, params![s.to_string()])
    } else {
        (conn.prepare(
            "SELECT id, command, args, status, timestamp, retry_count, max_retries, priority, schedule, is_recurring, next_run, parent_schedule_id, depends_on
            FROM jobs 
            ORDER BY timestamp ASC"
        )?, params![])
    };

    let jobs = stmt.query_map(param, |row| {
        let id_str: String = row.get(0)?;
        let command: String = row.get(1)?;
        let args_str: String = row.get(2)?;
        let status_str: String = row.get(3)?;
        let timestamp_str: String = row.get(4)?;

        let retry_cnt: u32 = row.get(5)?;
        let max_retry_cnt: u32 = row.get(6)?;

        let priority: String = row.get(7)?;

        let schedule: Option<String> = Some(row.get(8)?);
        let is_recurring: bool = row.get(9)?;
        let next_run_str: Option<String> = Some(row.get(10)?);
        let parent_id: Option<String> = row.get(11)?;

        let depends_on_str: String = row.get(12)?;

        let (schedule, is_recurring, next_run, p_id) = if schedule.as_deref() == Some("None") {
            (None, false, None, None)
        } else {
            (
                schedule,
                is_recurring,
                next_run_str.and_then(|s| DateTime::parse_from_rfc3339(&s).ok().map(|dt| dt.into())),
                parent_id.and_then(|s| Uuid::from_str(&s).ok())
            )
        };

        Ok(Job { 
            id: Uuid::from_str(&id_str).map_err(|_| Error::InvalidColumnType(0, id_str, Type::Text))?, 
            command, 
            args: serde_json::from_str::<Vec<String>>(&args_str).map_err(|_| Error::InvalidColumnType(2, args_str, Type::Text))?, 
            status: JobStatus::from_str(&status_str).map_err(|_| Error::InvalidColumnType(3, status_str, Type::Text))?, 
            timestamp: DateTime::parse_from_rfc3339(&timestamp_str).map_err(|_| Error::InvalidColumnType(4, timestamp_str, Type::Text))?.into(),
            
            retry_count: retry_cnt,
            max_retries: max_retry_cnt,

            priority: Priority::from_str(&priority).map_err(|_| Error::InvalidColumnType(7, priority, Type::Text))?,

            schedule,
            is_recurring: is_recurring,
            next_run,
            parent_schedule_id: p_id,

            depends_on: serde_json::from_str::<Option<Vec<Uuid>>>(&depends_on_str).map_err(|_| Error::InvalidColumnType(12, depends_on_str, Type::Text))?
        })
    })?;
    
    let results: Vec<Job> = jobs
        .filter_map(|v| match v {
            Ok(job) => Some(job),
            Err(e) => {
                log::warn!("Skipping malformed job in row: {}", e);
                None
            }
        })
        .collect();

    Ok(results)
}

pub fn load_pending_jobs(conn: &Connection) -> Result<Vec<Job>, Error> {
    let pending = fetch_from_db(conn, Some(JobStatus::PENDING));
    let running = fetch_from_db(conn, Some(JobStatus::RUNNING));

    if pending.is_ok() && running.is_ok() {
        Ok([pending.unwrap(), running.unwrap()].concat())
    } else if running.is_ok() {
        running
    } else {
        pending
    }

}

pub fn get_job_list(conn: &Connection, status: Option<JobStatus>) -> Result<Vec<Job>, Error> {
    let result = fetch_from_db(conn, status);

    result
}