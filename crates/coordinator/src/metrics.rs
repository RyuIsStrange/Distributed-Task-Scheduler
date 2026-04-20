use std::sync::LazyLock;
use prometheus::{Counter, CounterVec, register_counter, register_counter_vec};

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

