# Distributed Task Scheduler / Job Queue

## Overview
This is a distributed job scheduling system that allows you to submit, queue, and execute tasks across multiple worker nodes.

## Why I made this
I wanted to expand on skills that I have and wanted to make a proof of concept that is "Production Ready".

I put production ready in quotes as I am sure there are many things within this that wouldn't work or function well in a production environment.

However with this I have learned many things such as message design, database integration, job persistence and state management, and more.

I started this project ~1/5/26 and I am still working on this today.

## Technology Used
**Backend/Coordinator:**
- **Actix-web** - HTTP server for the REST API
    - **Actix-governor** - Rate limiting middleware for Actix-web, built on the token bucket algorithm
- **Tokio** - Async runtime for handling concurrent operations (worker checks, scheduled job polling, HTTP server)
- **rusqlite** - SQLite database for job persistence
- **cron** - Parsing and scheduling cron expressions
- **uuid** - Unique job identification
- **serde/serde_json** - JSON serialization
- **prometheus** - Metrics collection and exposition in the Prometheus text format for system observability

**Worker:**
- **Tokio** - Async runtime for job execution
- **reqwest** - Communication with coordinator (job polling, heartbeats, results)
- **tokio::process** - For executing shell commands/jobs

**CLI:**
- **Clap** (v4) - Command-line argument parsing with derive API
- **reqwest** - HTTP client for communicating with the coordinator
- **colored** - Pretty terminal output with colors for status indicators

**Shared:**
- **chrono** - Date/time handling
- **env_logger** - Logging across components

### What is new to me in this project?
- Large scale async with Tokio.
- Using SQLite or SQL in general.
- Cron as a whole.
- Clap for CLI commands.
- Chrono for life times of workers/ect.

## What am I working on?
CLI & DB Error Handling - Make errors easier for CLI clients to read, and make DB error less likely to malform or corrupt the DB. 

^ General Error fixes - This will be more of what limitations/deadlocks could the system encounter.

## What Works

**Job Management:**
- Submit, track, and retrieve results for jobs
- Three priority levels (High/Medium/Low)
- Retry failed jobs automatically (up to 3 times, configurable)
- Everything persists to SQLite

**Scheduling:**
- Cron syntax for recurring jobs
- Supports standard 5-field and extended 6-field expressions
- Scheduled jobs spawn regular jobs automatically

**Distributed Workers:**
- Multiple workers can pull jobs from the coordinator
- Heartbeat monitoring detects dead workers
- Jobs get recovered and re-queued if a worker dies

**CLI:**
- Submit jobs with `scheduler submit <command> --args "..." --priority <level> --schedule "cron expr"`
- Check status with `scheduler status <job-id>`
- List jobs with `scheduler list --status <filter>`
- Colored output to help visualize things.

**Job Dependencies:**
- Submit jobs with one or more dependency UUIDs using `--depends-on`
- Dependent jobs are blocked until all required jobs complete
- Invalid or nonexistent dependency UUIDs are rejected at submission with a 400 error
- If any dependency fails or is canceled, the dependent job is automatically marked as failed
- Blocked jobs are given a WAITING status so they are distinguishable from ready PENDING jobs

**Rate Limiting:**
- Job submission, status, and list endpoints are rate limited to 5 requests per second burst, replenishing at 1 request per 5 seconds
- Worker endpoints (heartbeat, job polling, result submission) are exempt from rate limiting
- Clients are informed with a clear message when rate limited (429 response)

**Metrics & Monitoring:**
- Prometheus-compatible /metrics endpoint exposed at the root level
- Six Prometheus metrics covering job submission, completion, failure, queue depth, worker count, and dependency-blocked jobs — all labeled by priority where applicable
- Worker and internal endpoints are excluded from rate limiting to ensure uninterrupted metric collection

### Whats next?