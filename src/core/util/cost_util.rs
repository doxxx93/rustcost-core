use crate::core::persistence::info::fixed::unit_price::info_unit_price_entity::InfoUnitPriceEntity;
use crate::domain::metric::k8s::common::service_helpers::BYTES_PER_GB;

pub struct CostUtil;

impl CostUtil {
    #[inline]
    pub fn bytes_to_gb(bytes: f64) -> f64 {
        bytes / BYTES_PER_GB
    }

    #[inline]
    pub fn bytes_to_gb_hours(bytes: f64, interval_hours: f64) -> f64 {
        Self::bytes_to_gb(bytes) * interval_hours
    }

    #[inline]
    pub fn compute_cpu_cost(nano_cores: f64, interval_hours: f64, prices: &InfoUnitPriceEntity) -> f64 {
        let cores = nano_cores / 1_000_000_000.0;
        let core_hours = cores * interval_hours;
        core_hours * prices.cpu_core_hour
    }

    #[inline]
    pub fn compute_memory_cost(bytes: f64, interval_hours: f64, prices: &InfoUnitPriceEntity) -> f64 {
        let gb_hours = Self::bytes_to_gb_hours(bytes, interval_hours);
        gb_hours * prices.memory_gb_hour
    }

    #[inline]
    pub fn compute_cpu_cost_from_core_nano_seconds(core_nano_seconds: f64, prices: &InfoUnitPriceEntity) -> f64 {
        // core_nano_seconds = core * nanoseconds
        // 1) to core-seconds: / 1e9
        // 2) to core-hours  : / 3600
        let core_hours = (core_nano_seconds / 1_000_000_000.0) / 3600.0;
        core_hours * prices.cpu_core_hour
    }
}
