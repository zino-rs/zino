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
            match self.description() {
                Some(description) => write!(f, "{name};desc={description};dur={duration}"),
                None => write!(f, "{name};dur={duration}"),
            }
        } else {
            match self.description() {
                Some(description) => write!(f, "{name};desc={description}"),
                None => write!(f, "{name}"),
            }
        }
    }
}
