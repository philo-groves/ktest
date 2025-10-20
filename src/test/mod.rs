use core::mem::MaybeUninit;
use crate::test::outcome::Termination;

pub mod output;
pub mod runner;
pub mod outcome;

/// A standard test.
///
/// This struct is created by the `#[test]` attribute. This struct is not to be used directly and
/// is not considered part of the public API. If you want to use a similar struct, you should
/// define one locally and implement `TestCase` for it directly.
#[doc(hidden)]
pub struct Test<T> {
    /// The test name.
    pub name: &'static str,
    /// The module path of the test.
    pub modules: &'static str,
    /// The test function itself.
    pub test: fn() -> T,
    /// Whether the test should be excluded. This is set by the `#[ignore]` attribute.
    pub ignore: Ignore,
    /// Whether the test is expected to panic. This is set by the `#[should_panic]` attribute.
    pub should_panic: ShouldPanic,
}

/// A trait representing a test case that can be run and provides metadata about itself.
pub trait TestCase {
    /// Returns the full name of the test case, including module path (e.g., "my_crate::tests::my_test").
    fn qualified_name(&self) -> &'static str;

    /// Returns the function name of the test case, without module path.
    fn name(&self) -> &'static str {
        let full_name = self.qualified_name();
        if let Some(pos) = full_name.rfind("::") {
            &full_name[pos + 2..]
        } else {
            full_name
        }
    }

    /// Returns the module path of the test case, if available.
    fn modules(&self) -> Option<&'static str> {
        let full_name = self.qualified_name();
        if let Some(pos) = full_name.rfind("::") {
            Some(&full_name[..pos])
        } else {
            None
        }
    }

    /// Runs the test case. This should not panic if the test passes.
    fn run(&self) -> ();

    /// Whether the test should be excluded or not.
    ///
    /// If this method returns `Ignore::Yes`, the test function will not be run at all (but it will
    /// still be compiled). This allows for time-consuming or expensive tests to be conditionally
    /// disabled.
    fn ignore(&self) -> Ignore;

    /// Whether the test is expected to panic.
    fn should_panic(&self) -> ShouldPanic;
}

impl<T> TestCase for Test<T>
where
    T: Termination,
{
    fn run(&self) {
        (self.test)().terminate();
    }
    
    fn qualified_name(&self) -> &'static str {
        self.name
    }

    fn name(&self) -> &'static str {
        self.name
    }

    fn modules(&self) -> Option<&'static str> {
        Some(self.modules)
    }

    fn ignore(&self) -> Ignore {
        self.ignore
    }

    fn should_panic(&self) -> ShouldPanic {
        self.should_panic
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Ignore {
    /// The test should be run.
    No,
    /// The test should not be run.
    Yes
}

#[derive(Clone, Copy, Debug)]
pub enum ShouldPanic {
    /// The test is expected to run successfully.
    No,
    /// The test is expected to panic during execution.
    Yes
}

#[doc(hidden)]
pub const fn split_module_path_len(module_path: &'static str) -> usize {
    let mut len = 1;

    let mut i = 1;
    while i < module_path.len() {
        if module_path.as_bytes()[i - 1] == b':' && module_path.as_bytes()[i] == b':' {
            len += 1;
            i += 1;
        }
        i += 1;
    }

    len
}

/// Splits a module path into its individual parts.
///
/// This function is used by the `#[test]` attribute. It is not considered a part of the public API.
#[doc(hidden)]
pub const fn split_module_path<const LEN: usize>(module_path: &'static str) -> [&'static str; LEN] {
    let mut result: MaybeUninit<[&'static str; LEN]> = MaybeUninit::uninit();
    let mut result_index = 0;
    let mut module_path_start = 0;
    // Look at two bytes at a time.
    let mut module_path_index = 1;
    while module_path_index < module_path.len() {
        if module_path.as_bytes()[module_path_index - 1] == b':'
            && module_path.as_bytes()[module_path_index] == b':'
        {
            let module = unsafe {
                str::from_utf8_unchecked(core::slice::from_raw_parts(
                    module_path.as_ptr().add(module_path_start),
                    module_path_index - 1 - module_path_start,
                ))
            };
            // Check that we have not already filled in the full result.
            if result_index >= LEN {
                panic!("module path was split into too many parts")
            }
            unsafe {
                (result.as_mut_ptr() as *mut &str)
                    .add(result_index)
                    .write(module);
            }
            result_index += 1;
            module_path_index += 1;
            module_path_start = module_path_index;
        }
        module_path_index += 1;
    }
    // Add the final path.
    let module = unsafe {
        str::from_utf8_unchecked(core::slice::from_raw_parts(
            module_path.as_ptr().add(module_path_start),
            module_path.len() - module_path_start,
        ))
    };
    // Check that we have not already filled in the full result.
    if result_index >= LEN {
        panic!("module path was split into too many parts")
    }
    unsafe {
        (result.as_mut_ptr() as *mut &str)
            .add(result_index)
            .write(module);
    }
    result_index += 1;

    // Check that we actually filled the result.
    if result_index < LEN {
        panic!("unable to split module path into enough separate parts")
    }

    unsafe { result.assume_init() }
}
