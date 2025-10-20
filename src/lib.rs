#![no_std]
#![feature(try_blocks)]
#![cfg_attr(test, no_main)]
#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, test_runner(runner))]
#![cfg_attr(test, reexport_test_harness_main = "test_harness")]

mod args;
mod log;
mod test;
mod qemu;

/// Re-export the test runner function for use in test binaries.
pub use test::runner::runner;

/// Maximum length for strings used in this library, to avoid dynamic allocations.
const MAX_STRING_LENGTH: usize = 1024;

/// Maximum length for large strings (e.g. output of all test names)
const MAX_STRING_LENGTH_LARGE: usize = MAX_STRING_LENGTH * 10;

/// Initialize the test harness with the given test group. This function should be called
/// before the main test function is called.
/// 
/// For example, in your lib.rs:
/// 
/// ```
/// ktest::init_harness("library");
/// test_main();
/// ```
/// 
pub fn init_harness(test_group: &str) {
    args::set_test_group(test_group);
}

/// A panic handler that delegates to the test runner's panic handler. This should be
/// included in libraries which use `ktest` to allow recovery from panics during tests.
/// 
/// Only include this in test builds.
/// 
/// For example, in your lib.rs:
/// 
/// ```
/// #[cfg(test)]
/// #[panic_handler]
/// fn panic(info: &core::panic::PanicInfo) -> ! {
///     ktest::panic(info)
/// }
/// ```
pub fn panic(info: &core::panic::PanicInfo) -> ! {
    use crate::test::runner::TestRunner;
    use crate::test::runner::TEST_RUNNER;

    TEST_RUNNER.get().unwrap().handle_panic(info)
}
