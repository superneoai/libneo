//! Supplies utility interfaces that GPUI imports.

use std::ffi::OsStr;
use std::future::Future;
use std::hash::{BuildHasher, Hasher};
use std::ops::AddAssign;
use std::panic::Location;
use std::pin::Pin;
use std::sync::OnceLock;
use std::task::{Context, Poll};
use std::time::Instant;

pub mod arc_cow;

/// Creates a command without opening an extra console window on Windows.
pub fn new_std_command(program: impl AsRef<OsStr>) -> std::process::Command {
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt as _;
        let mut command = std::process::Command::new(program);
        command.creation_flags(0x0800_0000);
        command
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::process::Command::new(program)
    }
}

/// Returns the configured Windows command shell.
#[cfg(target_os = "windows")]
pub fn get_windows_system_shell() -> String {
    std::env::var("COMSPEC").unwrap_or_else(|_| "cmd.exe".to_owned())
}

/// Increments a value and returns its prior value.
pub fn post_inc<T: From<u8> + AddAssign<T> + Copy>(value: &mut T) -> T {
    let previous = *value;
    *value += T::from(1);
    previous
}

/// Runs a closure and optionally reports its duration when `ZED_MEASUREMENTS` is enabled.
pub fn measure<R>(label: &str, function: impl FnOnce() -> R) -> R {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    let enabled = ENABLED.get_or_init(|| {
        std::env::var("ZED_MEASUREMENTS").is_ok_and(|value| value == "1" || value == "true")
    });

    if *enabled {
        let start = Instant::now();
        let result = function();
        eprintln!("{label}: {:?}", start.elapsed());
        result
    } else {
        function()
    }
}

/// Panics in debug builds and logs an error in release builds.
#[macro_export]
macro_rules! debug_panic {
    ($($argument:tt)*) => {{
        if cfg!(debug_assertions) {
            panic!($($argument)*);
        } else {
            log::error!($($argument)*);
        }
    }};
}

/// Reports an unexpected absent value in debug builds.
#[track_caller]
pub fn some_or_debug_panic<T>(value: Option<T>) -> Option<T> {
    if cfg!(debug_assertions) && value.is_none() {
        panic!("unexpected None");
    }
    value
}

/// Evaluates a block in a closure so it can use `?` independently of its caller.
#[macro_export]
macro_rules! maybe {
    ($block:block) => {
        (|| $block)()
    };
    (async $block:block) => {
        (async || $block)()
    };
    (async move $block:block) => {
        (async move || $block)()
    };
}

/// Logging and conversion helpers for results.
pub trait ResultExt<E> {
    /// The successful result type.
    type Ok;

    /// Logs an error and converts the result to an option.
    fn log_err(self) -> Option<Self::Ok>;

    /// Logs an error with debug formatting and converts the result to an option.
    fn log_err_with_backtrace(self) -> Option<Self::Ok>
    where
        E: std::fmt::Debug;

    /// Reports an error as a debug-only assertion and preserves the result.
    fn debug_assert_ok(self, reason: &str) -> Self;

    /// Logs an error as a warning and converts the result to an option.
    fn warn_on_err(self) -> Option<Self::Ok>;

    /// Logs an error at a selected level and converts the result to an option.
    fn log_with_level(self, level: log::Level) -> Option<Self::Ok>;

    /// Converts the error into an `anyhow` error.
    fn anyhow(self) -> anyhow::Result<Self::Ok>
    where
        E: Into<anyhow::Error>;
}

impl<T, E> ResultExt<E> for Result<T, E>
where
    E: std::fmt::Display,
{
    type Ok = T;

    #[track_caller]
    fn log_err(self) -> Option<T> {
        self.log_with_level(log::Level::Error)
    }

    #[track_caller]
    fn log_err_with_backtrace(self) -> Option<T>
    where
        E: std::fmt::Debug,
    {
        match self {
            Ok(value) => Some(value),
            Err(error) => {
                log_at(log::Level::Error, format_args!("{error:?}"));
                None
            }
        }
    }

    #[track_caller]
    fn debug_assert_ok(self, reason: &str) -> Self {
        if let Err(error) = &self {
            debug_panic!("{reason} - {error:#}");
        }
        self
    }

    #[track_caller]
    fn warn_on_err(self) -> Option<T> {
        self.log_with_level(log::Level::Warn)
    }

    #[track_caller]
    fn log_with_level(self, level: log::Level) -> Option<T> {
        match self {
            Ok(value) => Some(value),
            Err(error) => {
                log_at(level, format_args!("{error:#}"));
                None
            }
        }
    }

    fn anyhow(self) -> anyhow::Result<T>
    where
        E: Into<anyhow::Error>,
    {
        self.map_err(Into::into)
    }
}

#[track_caller]
fn log_at(level: log::Level, arguments: std::fmt::Arguments<'_>) {
    let caller = Location::caller();
    log::logger().log(
        &log::Record::builder()
            .args(arguments)
            .file(Some(caller.file()))
            .line(Some(caller.line()))
            .level(level)
            .build(),
    );
}

/// Logs a displayable error.
#[track_caller]
pub fn log_err<E: std::fmt::Display>(error: &E) {
    log_at(log::Level::Error, format_args!("{error:#}"));
}

/// Logging and unwrapping adapters for futures that return results.
pub trait TryFutureExt {
    /// Logs a failed future and resolves to an option.
    fn log_err(self) -> LogErrorFuture<Self>
    where
        Self: Sized;

    /// Logs a failed future using an explicitly tracked caller.
    fn log_tracked_err(self, location: Location<'static>) -> LogErrorFuture<Self>
    where
        Self: Sized;

    /// Warns for a failed future and resolves to an option.
    fn warn_on_err(self) -> LogErrorFuture<Self>
    where
        Self: Sized;

    /// Unwraps the result produced by the future.
    fn unwrap(self) -> UnwrapFuture<Self>
    where
        Self: Sized;
}

/// Debug-formatting adapters for futures that return results.
pub trait TryFutureExtBacktrace {
    /// Logs a failed future using debug formatting.
    fn log_err_with_backtrace(self) -> LogErrorWithBacktraceFuture<Self>
    where
        Self: Sized;

    /// Logs a failed future using debug formatting and an explicitly tracked caller.
    fn log_tracked_err_with_backtrace(
        self,
        location: Location<'static>,
    ) -> LogErrorWithBacktraceFuture<Self>
    where
        Self: Sized;
}

impl<F, T, E> TryFutureExt for F
where
    F: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    #[track_caller]
    fn log_err(self) -> LogErrorFuture<Self> {
        LogErrorFuture(self, log::Level::Error, *Location::caller())
    }

    fn log_tracked_err(self, location: Location<'static>) -> LogErrorFuture<Self> {
        LogErrorFuture(self, log::Level::Error, location)
    }

    #[track_caller]
    fn warn_on_err(self) -> LogErrorFuture<Self> {
        LogErrorFuture(self, log::Level::Warn, *Location::caller())
    }

    fn unwrap(self) -> UnwrapFuture<Self> {
        UnwrapFuture(self)
    }
}

impl<F, T, E> TryFutureExtBacktrace for F
where
    F: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    #[track_caller]
    fn log_err_with_backtrace(self) -> LogErrorWithBacktraceFuture<Self> {
        LogErrorWithBacktraceFuture(self, log::Level::Error, *Location::caller())
    }

    fn log_tracked_err_with_backtrace(
        self,
        location: Location<'static>,
    ) -> LogErrorWithBacktraceFuture<Self> {
        LogErrorWithBacktraceFuture(self, log::Level::Error, location)
    }
}

/// A future that logs errors and resolves to an option.
#[must_use]
pub struct LogErrorFuture<F>(F, log::Level, Location<'static>);

impl<F, T, E> Future for LogErrorFuture<F>
where
    F: Future<Output = Result<T, E>>,
    E: std::fmt::Display,
{
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        // The wrapper never moves its inner future after being pinned.
        let this = unsafe { self.get_unchecked_mut() };
        let inner = unsafe { Pin::new_unchecked(&mut this.0) };
        match inner.poll(context) {
            Poll::Ready(Ok(value)) => Poll::Ready(Some(value)),
            Poll::Ready(Err(error)) => {
                log::logger().log(
                    &log::Record::builder()
                        .args(format_args!("{error:#}"))
                        .file(Some(this.2.file()))
                        .line(Some(this.2.line()))
                        .level(this.1)
                        .build(),
                );
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A future that logs debug-formatted errors and resolves to an option.
#[must_use]
pub struct LogErrorWithBacktraceFuture<F>(F, log::Level, Location<'static>);

impl<F, T, E> Future for LogErrorWithBacktraceFuture<F>
where
    F: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    type Output = Option<T>;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        // The wrapper never moves its inner future after being pinned.
        let this = unsafe { self.get_unchecked_mut() };
        let inner = unsafe { Pin::new_unchecked(&mut this.0) };
        match inner.poll(context) {
            Poll::Ready(Ok(value)) => Poll::Ready(Some(value)),
            Poll::Ready(Err(error)) => {
                log::logger().log(
                    &log::Record::builder()
                        .args(format_args!("{error:?}"))
                        .file(Some(this.2.file()))
                        .line(Some(this.2.line()))
                        .level(this.1)
                        .build(),
                );
                Poll::Ready(None)
            }
            Poll::Pending => Poll::Pending,
        }
    }
}

/// A future that unwraps its result when ready.
pub struct UnwrapFuture<F>(F);

impl<F, T, E> Future for UnwrapFuture<F>
where
    F: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    type Output = T;

    fn poll(self: Pin<&mut Self>, context: &mut Context<'_>) -> Poll<Self::Output> {
        // The wrapper never moves its inner future after being pinned.
        let this = unsafe { self.get_unchecked_mut() };
        let inner = unsafe { Pin::new_unchecked(&mut this.0) };
        inner.poll(context).map(Result::unwrap)
    }
}

/// Runs a closure when dropped unless it has been aborted.
pub struct Deferred<F: FnOnce()>(Option<F>);

impl<F: FnOnce()> Deferred<F> {
    /// Prevents the deferred closure from running.
    pub fn abort(mut self) {
        self.0.take();
    }
}

impl<F: FnOnce()> Drop for Deferred<F> {
    fn drop(&mut self) {
        if let Some(function) = self.0.take() {
            function();
        }
    }
}

/// Defers a closure until the returned guard is dropped.
#[must_use]
pub fn defer<F: FnOnce()>(function: F) -> Deferred<F> {
    Deferred(Some(function))
}

/// A hash builder specialized for `TypeId` keys.
#[derive(Clone, Copy, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct TypeIdHashBuilder;

impl BuildHasher for TypeIdHashBuilder {
    type Hasher = TypeIdHasher;

    fn build_hasher(&self) -> Self::Hasher {
        TypeIdHasher::default()
    }
}

/// A hasher specialized for the native bytes written by `TypeId`.
#[derive(Clone, Copy, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct TypeIdHasher {
    value: u64,
}

impl Hasher for TypeIdHasher {
    fn write(&mut self, bytes: &[u8]) {
        let bytes: [u8; 8] = bytes
            .get(..8)
            .and_then(|bytes| bytes.try_into().ok())
            .expect("TypeId must write at least eight bytes");
        self.value = u64::from_ne_bytes(bytes);
    }

    fn finish(&self) -> u64 {
        self.value
    }
}

/// Retains the lowest `limit` values and sorts them with the supplied comparator.
pub fn truncate_to_bottom_n_sorted_by<T, F>(items: &mut Vec<T>, limit: usize, compare: &F)
where
    F: Fn(&T, &T) -> std::cmp::Ordering,
{
    items.sort_by(compare);
    items.truncate(limit);
}

#[cfg(test)]
mod tests {
    use super::{defer, post_inc, truncate_to_bottom_n_sorted_by};
    use std::cell::Cell;

    #[test]
    fn utility_contracts_hold() {
        let mut value = 4_u64;
        assert_eq!(post_inc(&mut value), 4);
        assert_eq!(value, 5);

        let called = Cell::new(false);
        {
            let _deferred = defer(|| called.set(true));
        }
        assert!(called.get());

        let mut values = vec![5, 1, 3, 2, 4];
        truncate_to_bottom_n_sorted_by(&mut values, 3, &Ord::cmp);
        assert_eq!(values, [1, 2, 3]);
    }
}
