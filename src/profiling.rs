#[cfg(all(feature = "profiling", not(feature = "optick-profiler")))]
macro_rules! internal_trace_span {
    ($name: literal) => {
        let span = tracing::trace_span!($name);
        let _enter = span.enter();
    };
}

#[cfg(all(feature = "profiling", not(feature = "optick-profiler")))]
macro_rules! internal_trace_future {
    ($f: expr) => {
        $f.in_current_span()
    };
}

#[cfg(all(feature = "profiling", not(feature = "optick-profiler")))]
macro_rules! internal_register_thread {
    ($name: literal) => {};
}

#[cfg(all(feature = "profiling", feature = "optick-profiler"))]
macro_rules! internal_trace_span {
    ($name: literal) => {
        optick::event!($name);
    };
}

#[cfg(all(feature = "profiling", feature = "optick-profiler"))]
macro_rules! internal_trace_future {
    ($f: expr) => {
        $f
    };
}

#[cfg(all(feature = "profiling", feature = "optick-profiler"))]
macro_rules! internal_register_thread {
    ($name: literal) => {
        optick::register_thread($name);
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

#[cfg(not(feature = "profiling"))]
macro_rules! internal_register_thread {
    ($name: literal) => {};
}
