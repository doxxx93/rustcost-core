use crate::domain::common::model::DaySplitRows;
use crate::domain::metric::k8s::common::service_helpers::TimeWindow;
use crate::domain::common::service::MetricRowRepository;
use anyhow::Result;
use chrono::NaiveTime;

pub fn split_day_granularity_rows<T>(
    object_name: &str,
    window: &TimeWindow,
    day_repo: &dyn MetricRowRepository<T>,
    hour_repo: &dyn MetricRowRepository<T>,
) -> Result<DaySplitRows<T>> {
    let start_date = window.start.date_naive();
    let end_date = window.end.date_naive();

    let is_start_full_day =
        window.start.time() == NaiveTime::from_hms_opt(0, 0, 0).unwrap();

    let is_end_full_day =
        window.end.time() >= NaiveTime::from_hms_opt(23, 59, 59).unwrap();

    // =========================
    // 1️⃣ start day → hour rows
    // =========================
    let start_hour_rows = if !is_start_full_day {
        let start_day_end = start_date
            .and_hms_opt(23, 59, 59)
            .unwrap()
            .and_utc();

        hour_repo.get_row_between(
            object_name,
            window.start,
            start_day_end.min(window.end),
        )?
    } else {
        vec![]
    };

    // =========================
    // 2️⃣ end day → hour rows
    // =========================
    let end_hour_rows = if start_date != end_date && !is_end_full_day {
        let end_day_start = end_date
            .and_hms_opt(0, 0, 0)
            .unwrap()
            .and_utc();

        hour_repo.get_row_between(
            object_name,
            end_day_start.max(window.start),
            window.end,
        )?
    } else {
        Vec::new()
    };

    // =========================
    // 3️⃣ middle full days → day rows
    // =========================
    let middle_start = if is_start_full_day {
        start_date
    } else {
        start_date.succ_opt().unwrap()
    };

    let middle_end = if is_end_full_day {
        end_date
    } else {
        end_date.pred_opt().unwrap()
    };

    let middle_day_rows = if middle_start <= middle_end {
        let middle_start_dt =
            middle_start.and_hms_opt(0, 0, 0).unwrap().and_utc();
        let middle_end_dt =
            middle_end.and_hms_opt(23, 59, 59).unwrap().and_utc();

        day_repo.get_row_between(
            object_name,
            middle_start_dt,
            middle_end_dt,
        )?
    } else {
        Vec::new()
    };

    Ok(DaySplitRows {
        start_hour_rows,
        end_hour_rows,
        middle_day_rows,
    })
}
