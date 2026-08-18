//! Supplies the tracing interface that GPUI imports.

pub use tracing::{
    Level, Span, debug_span, error_span, event, field, info_span, instrument, span, trace_span,
    warn_span,
};

/// Leaves tracing subscriber selection to the application.
pub fn init() {}
