//! Shared domain services/utils (e.g., cost calculator, time window logic)

pub(crate) mod day_granularity;

use anyhow::Result;
use chrono::{DateTime, Utc};

pub trait MetricRowRepository<T> {
    fn get_row_between(
        &self,
        object_name: &str,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<T>>;
}
