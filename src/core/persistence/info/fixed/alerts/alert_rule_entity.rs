use serde::{Deserialize, Serialize};

/// Metrics that can be evaluated by alert rules.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertMetricType {
    CpuUsagePercent,
    MemoryUsagePercent,
    DiskUsagePercent,
    GpuUsagePercent,
}

impl AlertMetricType {
    pub fn from_code<S: AsRef<str>>(code: S) -> Option<Self> {
        match code.as_ref().to_uppercase().as_str() {
            "CPU" => Some(Self::CpuUsagePercent),
            "MEMORY" => Some(Self::MemoryUsagePercent),
            "DISK" => Some(Self::DiskUsagePercent),
            "GPU" => Some(Self::GpuUsagePercent),
            _ => None,
        }
    }

    pub fn as_code(&self) -> &'static str {
        match self {
            Self::CpuUsagePercent => "CPU",
            Self::MemoryUsagePercent => "MEMORY",
            Self::DiskUsagePercent => "DISK",
            Self::GpuUsagePercent => "GPU",
        }
    }
}

/// Comparison operator for rule evaluation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertOperator {
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
}

impl AlertOperator {
    pub fn from_code<S: AsRef<str>>(code: S) -> Option<Self> {
        match code.as_ref().to_uppercase().as_str() {
            "GT" => Some(Self::GreaterThan),
            "LT" => Some(Self::LessThan),
            "GTE" => Some(Self::GreaterThanOrEqual),
            "LTE" => Some(Self::LessThanOrEqual),
            _ => None,
        }
    }

    pub fn as_code(&self) -> &'static str {
        match self {
            Self::GreaterThan => "GT",
            Self::LessThan => "LT",
            Self::GreaterThanOrEqual => "GTE",
            Self::LessThanOrEqual => "LTE",
        }
    }
}

/// Severity levels map to Discord embed colors.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

impl AlertSeverity {
    pub fn from_code<S: AsRef<str>>(code: S) -> Option<Self> {
        match code.as_ref().to_uppercase().as_str() {
            "INFO" => Some(Self::Info),
            "WARNING" => Some(Self::Warning),
            "CRITICAL" => Some(Self::Critical),
            _ => None,
        }
    }

    pub fn as_code(&self) -> &'static str {
        match self {
            Self::Info => "INFO",
            Self::Warning => "WARNING",
            Self::Critical => "CRITICAL",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AlertRuleEntity {
    pub id: String,
    pub name: String,
    pub metric_type: AlertMetricType,
    pub operator: AlertOperator,
    pub threshold: f64,
    pub for_duration_sec: u64,
    pub severity: AlertSeverity,
    pub enabled: bool,
}
