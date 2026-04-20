use prometheus::{Counter, CounterVec, Gauge, GaugeVec, register_counter, register_counter_vec, register_gauge, register_gauge_vec};
use std::sync::LazyLock;

pub static JOBS_COMPLETED_TOTAL: LazyLock<Counter> = LazyLock::new(|| {
    register_counter!(
        "jobs_completed_total",
        "Total number of completed jobs"
    ).unwrap()
});

pub static JOBS_SUBMITTED_TOTAL: LazyLock<CounterVec> = LazyLock::new(|| {
    register_counter_vec!(
        "jobs_submitted_total",
        "Total jobs submitted broken down by priority",
        &["priority"]
    ).unwrap()
});

pub static JOBS_FAILED_TOTAL: LazyLock<Counter> = LazyLock::new(||{
    register_counter!(
        "jobs_failed_total",
        "Total number of jobs that have failed"
    ).unwrap()
});

pub static QUEUE_DEPTH: LazyLock<GaugeVec> = LazyLock::new(|| {
    register_gauge_vec!(
        "queue_depth",
        "Current number of jobs pending broken down by priority",
        &["priority"]
    ).unwrap()
});

pub static ACTIVE_WORKERS: LazyLock<Gauge> = LazyLock::new(|| {
    register_gauge!(
        "active_workers",
        "Total of alive workers"
    ).unwrap()
});

pub static JOBS_WAITING_TOTAL: LazyLock<Gauge> = LazyLock::new(|| {
    register_gauge!(
        "jobs_waiting_total",
        "Total number of jobs blocked on dependencies"
    ).unwrap()
});