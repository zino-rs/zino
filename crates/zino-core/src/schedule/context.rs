use crate::{datetime::DateTime, Uuid};
use std::{any::Any, time::Instant};

/// Data associated with a job.
#[derive(Debug)]
pub struct JobContext {
    /// Job ID.
    job_id: Uuid,
    /// Job name.
    job_name: Option<&'static str>,
    /// Start time.
    start_time: Instant,
    /// Flag to indicate whether the job is disabled.
    disabled: bool,
    /// Flag to indicate whether the job is executed immediately.
    immediate: bool,
    /// Remaining ticks.
    remaining_ticks: Option<usize>,
    /// Last time when running the job.
    last_tick: Option<DateTime>,
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
            start_time: Instant::now(),
            disabled: false,
            immediate: false,
            remaining_ticks: None,
            last_tick: None,
            job_data: None,
        }
    }

    /// Starts the job.
    #[inline]
    pub fn start(&mut self) {
        self.start_time = Instant::now();
    }

    /// Finishes the job.
    #[inline]
    pub fn finish(&mut self) {
        if let Some(ticks) = self.remaining_ticks {
            self.remaining_ticks = Some(ticks.saturating_sub(1));
        }

        let job_id = self.job_id.to_string();
        let job_name = self.job_name;
        let execution_time = self.start_time.elapsed();
        tracing::warn!(
            job_id,
            job_name,
            remaining_ticks = self.remaining_ticks,
            last_tick = self.last_tick.map(|dt| dt.to_string()),
            execution_time_millis = execution_time.as_millis(),
        );
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
    }

    /// Sets the job name.
    #[inline]
    pub fn set_name(&mut self, name: &'static str) {
        self.job_name = Some(name);
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

    /// Returns `true` if the job is executed immediately.
    #[inline]
    pub fn is_immediate(&self) -> bool {
        self.immediate
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
}

impl Default for JobContext {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
