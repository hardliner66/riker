#[cfg(feature = "profiling")]
macro_rules! internal_trace_span {
    ($name: literal) => {
        let span = tracing::trace_span!($name);
        let _enter = span.enter();
    };
}

#[cfg(feature = "profiling")]
macro_rules! internal_trace_future {
    ($f: expr) => {
        $f.in_current_span()
    };
}

#[cfg(not(feature = "profiling"))]
macro_rules! internal_trace_span {
    ($name: literal) => {};
}

#[cfg(not(feature = "profiling"))]
macro_rules! internal_trace_future {
    ($f: expr) => {
        $f
    };
}
