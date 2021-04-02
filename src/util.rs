//! Helpers

/// `some_if(cond, f)` is `Some(f())` if `cond` is `true`, or else `None`
pub(crate) fn some_if<F, T>(cond: bool, f: F) -> Option<T>
where
    F: FnOnce() -> T,
{
    if !cond {
        None
    } else {
        Some(f())
    }
}
