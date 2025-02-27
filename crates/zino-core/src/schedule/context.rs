use crate::{Uuid, datetime::DateTime, error::Error};
use std::{
    any::Any,
    time::{Duration, Instant},
};

/// Data associated with a job.
#[derive(Debug)]
pub struct JobContext {
    /// Job ID.
    job_id: Uuid,
    /// Job name.
    job_name: Option<&'static str>,
    /// The source.
    source: String,
    /// The start time.
    start_time: Instant,
    /// Flag to indicate whether the job is disabled.
    disabled: bool,
    /// Flag to indicate whether the job is executed immediately.
    immediate: bool,
    /// Remaining ticks.
    remaining_ticks: Option<usize>,
    /// Last time when running the job.
    last_tick: Option<DateTime>,
    /// Next time when running the job.
    next_tick: Option<DateTime>,
    /// An error occurred in the job execution.
    execution_error: Option<Error>,
    /// Optional job data.
    job_data: Option<Box<dyn Any + Send>>,
}

impl JobContext {
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self {
            job_id: Uuid::now_v7(),
            job_name: None,
            source: String::new(),
            start_time: Instant::now(),
            disabled: false,
            immediate: false,
            remaining_ticks: None,
            last_tick: None,
            next_tick: None,
            execution_error: None,
            job_data: None,
        }
    }

    /// Starts the job.
    #[inline]
    pub fn start(&mut self) {
        self.start_time = Instant::now();
    }

    /// Finishes the job.
    pub fn finish(&mut self) {
        if let Some(ticks) = self.remaining_ticks {
            self.remaining_ticks = Some(ticks.saturating_sub(1));
        }

        let job_id = self.job_id.to_string();
        let job_name = self.job_name;
        let remaining_ticks = self.remaining_ticks;
        let last_tick = self.last_tick.map(|dt| dt.to_string());
        let next_tick = self.next_tick.map(|dt| dt.to_string());
        let execution_time = self.start_time.elapsed();
        let execution_time_millis = execution_time.as_millis();
        if let Some(error) = self.execution_error.as_ref() {
            tracing::error!(
                job_id,
                job_name,
                remaining_ticks,
                last_tick,
                next_tick,
                execution_time_millis,
                "{error}"
            );
        } else {
            tracing::warn!(
                job_id,
                job_name,
                remaining_ticks,
                last_tick,
                next_tick,
                execution_time_millis,
            );
        }
        #[cfg(feature = "metrics")]
        if let Some(name) = job_name {
            metrics::histogram!(
                "zino_job_execution_duration_seconds",
                "job_name" => name,
            )
            .record(execution_time.as_secs_f64());
        } else {
            metrics::histogram!(
                "zino_job_execution_duration_seconds",
                "job_id" => job_id,
            )
            .record(execution_time.as_secs_f64());
        }
        self.set_last_tick(DateTime::now());
    }

    /// Records an error occurred in the job execution.
    #[inline]
    pub fn record_error(&mut self, error: impl Into<Error>) {
        self.execution_error = Some(error.into());
    }

    /// Retries to run the job after a time span.
    #[inline]
    pub fn retry_after(&mut self, duration: Duration) {
        self.next_tick = Some(DateTime::now() + duration);
    }

    /// Sets the job name.
    #[inline]
    pub fn set_name(&mut self, name: &'static str) {
        self.job_name = Some(name);
    }

    /// Sets the source.
    #[inline]
    pub fn set_source(&mut self, source: impl Into<String>) {
        self.source = source.into();
    }

    /// Sets the remaining_ticks.
    #[inline]
    pub fn set_remaining_ticks(&mut self, ticks: usize) {
        self.remaining_ticks = Some(ticks);
    }

    /// Sets the last tick.
    #[inline]
    pub fn set_last_tick(&mut self, last_tick: DateTime) {
        self.last_tick = Some(last_tick);
    }

    /// Sets the next tick.
    #[inline]
    pub fn set_next_tick(&mut self, next_tick: Option<DateTime>) {
        self.next_tick = next_tick;
    }

    /// Sets the disabled status.
    #[inline]
    pub fn set_disabled_status(&mut self, disabled: bool) {
        self.disabled = disabled;
    }

    /// Sets the immediate mode.
    #[inline]
    pub fn set_immediate_mode(&mut self, immediate: bool) {
        self.immediate = immediate;
    }

    /// Sets the job data.
    #[inline]
    pub fn set_data<T: Send + 'static>(&mut self, data: T) {
        self.job_data = Some(Box::new(data));
    }

    /// Gets a reference to the job data.
    #[inline]
    pub fn get_data<T: Send + 'static>(&self) -> Option<&T> {
        self.job_data
            .as_ref()
            .and_then(|data| data.downcast_ref::<T>())
    }

    /// Gets a mutable reference to the job data.
    #[inline]
    pub fn get_data_mut<T: Send + 'static>(&mut self) -> Option<&mut T> {
        self.job_data
            .as_mut()
            .and_then(|data| data.downcast_mut::<T>())
    }

    /// Takes the job data out of the context.
    #[inline]
    pub fn take_data<T: Send + 'static>(&mut self) -> Option<Box<T>> {
        self.job_data
            .take()
            .and_then(|data| data.downcast::<T>().ok())
    }

    /// Returns the Job ID.
    #[inline]
    pub fn job_id(&self) -> Uuid {
        self.job_id
    }

    /// Returns the job name.
    #[inline]
    pub fn job_name(&self) -> Option<&'static str> {
        self.job_name
    }

    /// Returns a reference to the source, *i.e.* the cron expression for a cron job.
    #[inline]
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Returns the start time.
    #[inline]
    pub fn start_time(&self) -> Instant {
        self.start_time
    }

    /// Returns `true` if the job is disabled.
    #[inline]
    pub fn is_disabled(&self) -> bool {
        self.disabled
    }

    /// Returns `true` if the job should be executed immediately.
    pub fn is_immediate(&self) -> bool {
        self.immediate && self.last_tick.is_none()
            || self.next_tick.and_then(|dt| dt.span_before_now()).is_some()
    }

    /// Returns `true` if the job is fused and can not be executed any more.
    #[inline]
    pub fn is_fused(&self) -> bool {
        self.remaining_ticks == Some(0)
    }

    /// Returns the last tick.
    #[inline]
    pub fn last_tick(&self) -> Option<DateTime> {
        self.last_tick
    }

    /// Returns the next tick.
    #[inline]
    pub fn next_tick(&self) -> Option<DateTime> {
        self.next_tick
    }

    /// Returns the execution error.
    #[inline]
    pub fn execution_error(&self) -> Option<&Error> {
        self.execution_error.as_ref()
    }
}

impl Default for JobContext {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
