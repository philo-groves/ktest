use alloc::{string::{String, ToString}, vec::Vec};
use crate::{debugcon_println, test::TestCase};

/// Writes a JSON object indicating the start of a test group with its name and test count.
pub fn write_test_group(test_group: &str, test_count: usize) {
    let test_group_json = serde_json::json!({ 
        "test_group": test_group,
        "test_count": test_count
    }).to_string().replace("\n", "");
    debugcon_println!("{}", test_group_json);
}

/// Writes a JSON array of all test names. Helps to verify that all tests were ran in the case of a crash.
pub fn write_test_names(tests: &'static [&'static dyn TestCase]) {
    let test_names: Vec<String> = tests.iter().map(|t| t.name().to_string()).collect();
    let test_names_json = serde_json::json!({ "tests": test_names }).to_string().replace("\n", "");
    debugcon_println!("{}", test_names_json);
}

/// Writes a JSON object indicating the success of a test case, including its name and cycle count.
pub fn write_test_success(test_name: &str, cycle_count: u64) {
    let test_json = serde_json::json!({ "test": test_name, "result": "ok", "cycle_count": cycle_count })
        .to_string()
        .replace("\n", "");
    debugcon_println!("{}", test_json);
}

/// Writes a JSON object indicating the failure of a test case, including its name, location, and failure message.
pub fn write_test_failure(test_name: &str, location: &str, message: &str) {
    let test_json = serde_json::json!({ 
        "test": test_name, 
        "result": "fail", 
        "cycle_count": 0, // no way to track this for failure (yet?)
        "location": location,
        "message": message
    })
    .to_string()
    .replace("\n", "");
    debugcon_println!("{}", test_json);
}
