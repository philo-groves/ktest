pub mod output;
pub mod runner;

/// A trait representing a test case that can be run and provides metadata about itself.
pub trait TestCase {
    /// Runs the test case. This should not panic if the test passes.
    fn run(&self) -> ();

    /// Returns the full name of the test case, including module path (e.g., "my_crate::tests::my_test").
    fn name(&self) -> &'static str;

    /// Returns the module path of the test case, if available.
    fn module_path(&self) -> Option<&'static str> {
        let full_name = self.name();
        if let Some(pos) = full_name.rfind("::") {
            Some(&full_name[..pos])
        } else {
            None
        }
    }

    /// Returns the function name of the test case, without module path.
    fn function_name(&self) -> &'static str {
        let full_name = self.name();
        if let Some(pos) = full_name.rfind("::") {
            &full_name[pos + 2..]
        } else {
            full_name
        }
    }
}

impl<T> TestCase for T where T: Fn() {
    fn run(&self) {
        self();
    }

    fn name(&self) -> &'static str {
        core::any::type_name::<T>()
    }
}
