use crate::SharedString;
use smallvec::SmallVec;
use std::fmt;

/// A record of vendor-specific trace data across tracing systems.
#[derive(Debug, Clone)]
pub struct TraceState {
    /// Vendor-specific trace state.
    states: SmallVec<[(SharedString, String); 2]>,
}

impl TraceState {
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self {
            states: SmallVec::new(),
        }
    }

    /// Constructs an instance from the `tracestate` header value.
    pub fn from_tracestate(tracestate: &str) -> Self {
        let states = tracestate
            .replace(' ', "")
            .split(',')
            .filter_map(|state| {
                state
                    .split_once('=')
                    .map(|(key, value)| (key.to_owned().into(), value.to_owned()))
            })
            .collect();
        Self { states }
    }

    /// Pushes a key-value pair into the list of states. If an entry with the key already exists,
    /// the value will be updated.
    #[inline]
    pub fn push(&mut self, key: impl Into<SharedString>, value: impl ToString) {
        let states = &mut self.states;
        let key = key.into();
        let value = value.to_string();
        if let Some(index) = states.iter().position(|(k, _)| k.as_ref() == key.as_ref()) {
            states[index] = (key, value);
        } else {
            states.push((key, value));
        }
    }
}

impl Default for TraceState {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TraceState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let output = self
            .states
            .iter()
            .map(|(key, value)| format!("{key}={value}"))
            .collect::<Vec<_>>()
            .join(",");
        write!(f, "{output}")
    }
}
