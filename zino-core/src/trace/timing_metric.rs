use crate::SharedString;
use std::{fmt, time::Duration};

/// A metric of the performance timing.
#[derive(Debug, Clone)]
pub struct TimingMetric {
    /// Metric name.
    name: SharedString,
    /// Optional description.
    description: Option<SharedString>,
    /// Timing duration. A zero value means that it does not exist.
    duration: Duration,
}

impl TimingMetric {
    /// Creates a new instance.
    #[inline]
    pub fn new(
        name: SharedString,
        description: Option<SharedString>,
        duration: Option<Duration>,
    ) -> Self {
        Self {
            name,
            description,
            duration: duration.unwrap_or_default(),
        }
    }

    /// Returns the name.
    #[inline]
    pub fn name(&self) -> &str {
        self.name.as_ref()
    }

    /// Returns the description.
    #[inline]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Returns the timing duration.
    #[inline]
    pub fn duration(&self) -> Option<Duration> {
        let duration = self.duration;
        (duration > Duration::ZERO).then_some(duration)
    }
}

impl fmt::Display for TimingMetric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = self.name();
        if let Some(duration) = self.duration() {
            let duration_millis = format!("{:.3}", duration.as_secs_f64() * 1000.0);
            let duration = duration_millis.trim_end_matches(['.', '0']);
            if let Some(description) = self.description() {
                write!(f, "{name};desc={description};dur={duration}")
            } else {
                write!(f, "{name};dur={duration}")
            }
        } else if let Some(description) = self.description() {
            write!(f, "{name};desc={description}")
        } else {
            write!(f, "{name}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TimingMetric;
    use std::time::Duration;

    #[test]
    fn it_formats_timing_metric() {
        let cache_miss_metric = TimingMetric::new("miss".into(), None, None);
        assert_eq!(format!("{cache_miss_metric}"), "miss");

        let db_query_metric = TimingMetric::new(
            "db".into(),
            Some("query".into()),
            Some(Duration::from_secs_f64(0.0024635)),
        );
        assert_eq!(format!("{db_query_metric}"), "db;desc=query;dur=2.463");

        let total_timing_metric =
            TimingMetric::new("total".into(), None, Some(Duration::from_secs_f64(0.01082)));
        assert_eq!(format!("{total_timing_metric}"), "total;dur=10.82");
    }
}
