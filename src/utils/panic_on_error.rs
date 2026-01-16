use anyhow::Result;

/// Panics if passed function returns `Result::Err`.
///
/// This is just convenience utility wrapper for Criterion benchmarks, where
/// - you don't want to clutter code with `expect()` or `unwrap()`
/// - you cannot return Result, since Criterion ignores value returned from
///   benchmark, which triggers `not_used` lint warning
pub fn panic_on_error(f: impl FnOnce() -> Result<()>) {
    #[expect(clippy::unwrap_used)]
    f().unwrap();
}
