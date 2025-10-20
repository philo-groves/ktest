use conquer_once::spin::OnceCell;
use heapless::{format, String};
use spin::RwLock;
use crate::{args, qemu, serial_print, serial_println, test::{self, outcome::TestResult, Ignore, ShouldPanic, TestCase}, MAX_STRING_LENGTH};

/// A static reference to the list of test functions to run. This is unsafe but only set 
/// once at the start of runner. The static nature of the tests makes it impossible to use 
/// OnceCell, Mutex, or RwLock here (at least their no_std variants).
static mut TESTS: &'static [&'static dyn TestCase] = &[];

/// The global test runner instance. This is initialized once at the start of runner.
pub static TEST_RUNNER: OnceCell<KernelTestRunner> = OnceCell::uninit();

/// Tracker for the curent test index (corresponding to the index in TESTS)
pub static CURRENT_TEST_INDEX: OnceCell<RwLock<usize>> = OnceCell::new(RwLock::new(0));

/// Tracker for the current module name, to print headers when it changes
pub static CURRENT_MODULE: OnceCell<RwLock<&'static str>> = OnceCell::new(RwLock::new(""));

/// A test runner that runs the given tests and exits QEMU after completion.
/// 
/// Output from this runner is formatted as line-delimited JSON and printed to the debug 
/// console. This allows for easy parsing of test results by external tools, such as `kboot`.
pub fn runner(tests: &'static [&'static dyn TestCase]) -> ! {
    unsafe { TESTS = tests; }

    TEST_RUNNER.get_or_init(|| KernelTestRunner::default());
    TEST_RUNNER.get().unwrap().run_tests(0)
}

/// A trait defining the behavior of a test runner.
pub trait TestRunner {
    /// Runs once before all tests.
    fn before_tests(&self);
    /// Runs all tests starting from the given index.
    fn run_tests(&self, start_index: usize) -> !;
    /// Runs once after all tests.
    fn after_tests(&self) -> !;
    /// Called at the start of each test, returns the starting cycle number.
    fn start_test(&self) -> u64;
    /// Called at the end of each test, with the result and starting cycle number.
    fn complete_test(&self, result: TestResult, cycle_start: u64);
    /// Returns the currently running test, if any.
    fn current_test(&self) -> Option<&'static dyn TestCase>;
    /// Called when a test panics. This should print the panic information, mark the current
    /// test as failed, and continue with the next test (if possible).
    fn handle_panic(&self, info: &core::panic::PanicInfo) -> !;
}

/// A kernel test runner that runs all tests sequentially and exits QEMU after completion.
#[derive(Default)]
pub struct KernelTestRunner;

impl TestRunner for KernelTestRunner {
    fn before_tests(&self) {
        let test_group = args::get_test_group().unwrap_or("default");
        let tests = unsafe { TESTS };

        test::output::write_test_group(test_group, tests.len());
        test::output::write_test_names(tests);
    }

    fn run_tests(&self, start_index: usize) -> ! {
        if start_index == 0 { // dont run before_tests if resuming
            self.before_tests();
        }
        
        let tests = unsafe { TESTS };
        for (i, &test) in tests.iter().enumerate().skip(start_index) {
            let cycle_start = self.start_test();

            match test.ignore() {
                Ignore::No => {
                    test.run();
                    self.complete_test(TestResult::Success, cycle_start);
                }
                Ignore::Yes => {
                    self.complete_test(TestResult::Ignored, cycle_start);
                }
            }

            if !increment_test_index(i) {
                break; // no more tests to run
            }
        }
        self.after_tests()
    }

    fn after_tests(&self) -> ! {
        qemu::exit(qemu::ExitCode::Success)
    }

    fn start_test(&self) -> u64 {
        // check if the current module has changed; if so, reassign it and print a header
        let current_test = self.current_test().unwrap();
        
        let module_path = current_test.modules().unwrap_or("unknown_module");
        {
            let mut current_module = CURRENT_MODULE.get().unwrap().write();
            if *current_module != module_path {
                *current_module = module_path;

                let module_test_count = count_by_module(module_path);
                let test_group = args::get_test_group().unwrap_or("default");
                serial_println!("\n################################################################");
                serial_println!("# Running {} {} tests for module: {}", module_test_count, test_group, module_path);
                serial_println!("----------------------------------------------------------------");
            }
        } // scope will release the lock here

        // print the test name with padding for aligned results
        print_test_name(current_test.name(), 58);

        // return the current cycle (for duration calculation later)
        read_current_cycle()
    }

    fn complete_test(&self, result: TestResult, cycle_start: u64) {
        let cycle_count = if cycle_start != u64::MAX { // u64::MAX = unknown
            read_current_cycle() - cycle_start
        } else {
            0
        };

        match result {
            TestResult::Success => {
                let test_name = self.current_test().unwrap().qualified_name();
                test::output::write_test_success(test_name, cycle_count);
                serial_println!("[pass]");
            }
            TestResult::Failure => {
                // panic handler will print [fail] with details (and same for JSON output)
            }
            TestResult::Ignored => {
                let test_name = self.current_test().unwrap().qualified_name();
                test::output::write_test_ignored(test_name);
                serial_println!("[ignored]");
            }
        }
    }

    fn current_test(&self) -> Option<&'static dyn TestCase> {
        let current_index = *CURRENT_TEST_INDEX.get().unwrap().read();
        let tests = unsafe { TESTS };
        tests.get(current_index).copied()
    }

    fn handle_panic(&self, info: &core::panic::PanicInfo) -> ! {
        // finish the test output, replaces [pass] with panic details
        let location = if let Some(location) = info.location() {
            format!("{}:{}", location.file(), location.line()).unwrap()
        } else {
            String::<MAX_STRING_LENGTH>::try_from("unknown location").unwrap()
        };
        let message = info.message().as_str().unwrap_or("no message");

        let current_test = self.current_test().unwrap();
        let test_name = current_test.qualified_name();

        // handle according to whether the test was expected to panic
        match current_test.should_panic() {
            ShouldPanic::No => {
                serial_println!("[fail] @ {}: {}", location, message); // expected that the line already has "test_name... "
                test::output::write_test_failure(test_name, location.as_str(), message);
                self.complete_test(TestResult::Failure, u64::MAX);
            }
            ShouldPanic::Yes => {
                test::output::write_test_success(test_name, 0);
                self.complete_test(TestResult::Success, u64::MAX);
            }
        }

        // increment the test index to move to the next test (if possible)
        let current_index = *CURRENT_TEST_INDEX.get().unwrap().read();
        if !increment_test_index(current_index) {
            qemu::exit(qemu::ExitCode::Success); // no more tests to run
        }

        // continue with the next test (and all thereafter)
        self.run_tests(current_index + 1) // continue with next test
    }
}

/// Helper function to read the current CPU cycle count using the RDTSC instruction.
fn read_current_cycle() -> u64 {
    unsafe { core::arch::x86_64::_rdtsc() }
}

/// Helper function to assign base + 1 to CURRENT_TEST_INDEX.
fn increment_test_index(base: usize) -> bool {
    let mut current_test_index = CURRENT_TEST_INDEX.get().unwrap().write();
    
    let tests = unsafe { TESTS };
    if *current_test_index >= tests.len() {
        return false; // no more tests to run
    }
    
    *current_test_index = base + 1;
    true
}

/// Helper function to count the number of tests in a given module.
fn count_by_module(module_name: &str) -> usize {
    let tests = unsafe { TESTS };
    tests.iter()
        .filter(|&&test| test.modules().unwrap_or("") == module_name)
        .count()
}

/// Helper to write function names with padding for aligned results
fn print_test_name(name: &str, result_column: usize) {
    if name.len() >= result_column {
        serial_print!("{} ", name); // no padding if name is too long
        return;
    }

    let padding = result_column - name.len();
    serial_print!("{}", name);
    for _ in 0..padding {
        serial_print!(" ");
    }
}
