use super::TraceState;
use crate::{extension::HeaderMapExt, Uuid};
use http::header::HeaderMap;
use tracing::Span;

/// The `sampled` flag.
const FLAG_SAMPLED: u8 = 1;

///The `random-trace-id` flag.
const FLAG_RANDOM_TRACE_ID: u8 = 2;

/// HTTP headers for distributed tracing.
/// See [the spec](https://w3c.github.io/trace-context).
#[derive(Debug, Clone)]
pub struct TraceContext {
    /// Span identifier.
    span_id: u64,
    /// Version of the traceparent header.
    version: u8,
    /// Globally unique identifier.
    trace_id: u128,
    /// Identifier of the request known by the caller.
    parent_id: Option<u64>,
    /// Trace flags.
    trace_flags: u8,
    /// Trace state.
    trace_state: TraceState,
}

impl TraceContext {
    /// Creates a new instance without parent.
    pub fn new() -> Self {
        let span_id = Span::current()
            .id()
            .map(|id| id.into_u64())
            .unwrap_or_else(rand::random);
        Self {
            span_id,
            version: 0,
            trace_id: Uuid::now_v7().as_u128(),
            parent_id: None,
            trace_flags: FLAG_SAMPLED | FLAG_RANDOM_TRACE_ID,
            trace_state: TraceState::new(),
        }
    }

    /// Creates a new instance with the specific `trace-id`.
    pub fn with_trace_id(trace_id: Uuid) -> Self {
        let span_id = Span::current()
            .id()
            .map(|id| id.into_u64())
            .unwrap_or_else(rand::random);
        Self {
            span_id,
            version: 0,
            trace_id: trace_id.as_u128(),
            parent_id: None,
            trace_flags: FLAG_SAMPLED | FLAG_RANDOM_TRACE_ID,
            trace_state: TraceState::new(),
        }
    }

    /// Creates a child of the current trace context.
    pub fn child(&self) -> Self {
        let span_id = Span::current()
            .id()
            .map(|id| id.into_u64())
            .unwrap_or_else(rand::random);
        Self {
            span_id,
            version: self.version,
            trace_id: self.trace_id,
            parent_id: Some(self.span_id),
            trace_flags: self.trace_flags,
            trace_state: self.trace_state.clone(),
        }
    }

    /// Constructs an instance from the `traceparent` header value.
    pub fn from_traceparent(traceparent: &str) -> Option<Self> {
        let span_id = Span::current()
            .id()
            .map(|id| id.into_u64())
            .unwrap_or_else(rand::random);
        let parts = traceparent.split('-').collect::<Vec<_>>();
        (parts.len() == 4).then_some(Self {
            span_id,
            version: u8::from_str_radix(parts[0], 16).ok()?,
            trace_id: u128::from_str_radix(parts[1], 16).ok()?,
            parent_id: Some(u64::from_str_radix(parts[2], 16).ok()?),
            trace_flags: u8::from_str_radix(parts[3], 16).ok()?,
            trace_state: TraceState::new(),
        })
    }

    /// Constructs an instance from the `traceparent` and `tracestate` header values.
    pub fn from_headers(headers: &HeaderMap) -> Option<Self> {
        let traceparent = headers.get_str("traceparent")?;
        let mut trace_context = Self::from_traceparent(traceparent)?;
        if let Some(tracestate) = headers.get_str("tracestate") {
            trace_context.trace_state = TraceState::from_tracestate(tracestate);
        }
        Some(trace_context)
    }

    /// Returns the `span-id`.
    #[inline]
    pub fn span_id(&self) -> u64 {
        self.span_id
    }

    /// Returns the `version` of the spec used.
    #[inline]
    pub fn version(&self) -> u8 {
        self.version
    }

    /// Returns the `trace-id`.
    #[inline]
    pub fn trace_id(&self) -> u128 {
        self.trace_id
    }

    /// Returns the `parent-id`.
    #[inline]
    pub fn parent_id(&self) -> Option<u64> {
        self.parent_id
    }

    /// Returns the `trace-flags`.
    #[inline]
    pub fn trace_flags(&self) -> u8 {
        self.trace_flags
    }

    /// Returns true if the `sampled` flag has been enabled.
    #[inline]
    pub fn sampled(&self) -> bool {
        (self.trace_flags & FLAG_SAMPLED) == FLAG_SAMPLED
    }

    /// Returns true if the `random-trace-id` flag has been enabled.
    #[inline]
    pub fn random_trace_id(&self) -> bool {
        (self.trace_flags & FLAG_RANDOM_TRACE_ID) == FLAG_RANDOM_TRACE_ID
    }

    /// Sets the `sampled` flag.
    #[inline]
    pub fn set_sampled(&mut self, sampled: bool) {
        self.trace_flags ^= ((sampled as u8) ^ self.trace_flags) & FLAG_SAMPLED;
    }

    /// Sets the `random-trace-id` flag.
    #[inline]
    pub fn set_random_trace_id(&mut self, random: bool) {
        self.trace_flags ^= ((random as u8) ^ self.trace_flags) & FLAG_RANDOM_TRACE_ID;
    }

    /// Returns a mutable reference to the trace state.
    #[inline]
    pub fn trace_state_mut(&mut self) -> &mut TraceState {
        &mut self.trace_state
    }

    /// Formats the `traceparent` header value.
    #[inline]
    pub fn traceparent(&self) -> String {
        format!(
            "{:02x}-{:032x}-{:016x}-{:02x}",
            self.version, self.trace_id, self.span_id, self.trace_flags
        )
    }

    /// Formats the `tracestate` header value.
    #[inline]
    pub fn tracestate(&self) -> String {
        self.trace_state.to_string()
    }
}

impl Default for TraceContext {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::TraceContext;

    #[test]
    fn it_constructs_trace_context() {
        let traceparent = "00-76580b47d0bf430ebbb0d1d966b10f2b-0000004000000001-03";
        let trace_context = TraceContext::from_traceparent(traceparent).unwrap();
        assert_eq!(trace_context.version(), 0);
        assert_eq!(
            trace_context.trace_id(),
            u128::from_str_radix("76580b47d0bf430ebbb0d1d966b10f2b", 16).unwrap(),
        );
        assert_eq!(
            trace_context.parent_id(),
            u64::from_str_radix("0000004000000001", 16).ok(),
        );
        assert_eq!(trace_context.trace_flags(), 3);
    }
}
