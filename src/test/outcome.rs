//! Partial credit to: https://github.com/Anders429/gba_test/blob/master/gba_test/src/termination.rs
//! Defines a `Termination` trait to allow for different return types on `#[test]` definitions.

use core::fmt::Debug;

/// A trait for implementing arbitrary return types for [`#[test]`](crate::test)s.
///
/// This trait is analogous to the standard library's
/// [`Termination`](https://doc.rust-lang.org/std/process/trait.Termination.html). The main
/// difference is that rather than returning an exit code, this trait simply returns `()` on
/// success and panics on failure.
///
/// This trait is implemented for `()`, which covers the standard `#[test]` definition, and on
/// `Result<T, E>`, which allows for tests with signatures like `fn foo() -> Result<(), E>`.
pub trait Termination {
    /// Called to determine whether the test result is a success or a failure.
    ///
    /// On success, this should do nothing. On failure, it should panic.
    fn terminate(self);
}

impl Termination for () {
    fn terminate(self) {}
}

impl<T, E> Termination for Result<T, E>
where
    T: Termination,
    E: Debug,
{
    fn terminate(self) {
        match self {
            Ok(value) => value.terminate(),
            Err(error) => panic!("{error:?}"),
        }
    }
}

pub enum TestResult {
    Success,
    Failure,
    Ignore
}

impl TestResult {
    pub fn is_success(&self) -> bool {
        matches!(self, TestResult::Success)
    }

    pub fn is_ignore(&self) -> bool {
        matches!(self, TestResult::Ignore)
    }

    pub fn is_failure(&self) -> bool {
        matches!(self, TestResult::Failure)
    }
}
