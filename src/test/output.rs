use heapless::{format, String};
use crate::{debugcon_println, test::TestCase, MAX_STRING_LENGTH, MAX_STRING_LENGTH_LARGE};

/// Writes a JSON object indicating the start of a test group with its name and test count.
pub fn write_test_group(test_group: &str, test_count: usize) {
    let test_group_json: String<MAX_STRING_LENGTH> = format!(r#"{{ "test_group": "{}", "test_count": {} }}"#, 
        test_group, test_count).unwrap();
    let test_group_json = replace_heapless_string(&test_group_json, "\n", "").unwrap();
    let test_group_json = replace_heapless_string(&test_group_json, "\t", "").unwrap();

    debugcon_println!("{}", test_group_json);
}

/// Writes a JSON array of all test names. Helps to verify that all tests were ran in the case of a crash.
pub fn write_test_names(tests: &'static [&'static dyn TestCase]) {
    let mut test_names_str: String<MAX_STRING_LENGTH_LARGE> = String::try_from(r#"{ "tests": ["#).unwrap();
    for (i, test) in tests.iter().enumerate() {
        let entry: String<MAX_STRING_LENGTH> = if i == tests.len() - 1 {
            format!(r#""{}::{}""#, test.modules().unwrap(), test.name()).unwrap()
        } else {
            format!(r#""{}::{}", "#, test.modules().unwrap(), test.name()).unwrap()
        };
        test_names_str.push_str(&entry).unwrap();
    }
    test_names_str.push_str("] }").unwrap();
    debugcon_println!("{}", test_names_str);
}

/// Writes a JSON object indicating the success of a test case, including its name and cycle count.
pub fn write_test_success(test_name: &str, cycle_count: u64) {
    let test_json: String<MAX_STRING_LENGTH> = format!(r#"
{{
    "test": "{}",
    "result": "pass",
    "cycle_count": {}
}}"#, test_name, cycle_count).unwrap();
    let test_json = replace_heapless_string(&test_json, "\n", "").unwrap();
    let test_json = replace_heapless_string(&test_json, "   ", "").unwrap();

    debugcon_println!("{}", test_json);
}

/// Writes a JSON object indicating the ignore of a test case, including its name and cycle count.
pub fn write_test_ignore(test_name: &str) {
    let test_json: String<MAX_STRING_LENGTH> = format!(r#"
{{
    "test": "{}",
    "result": "ignore",
    "cycle_count": 0
}}"#, test_name).unwrap();
    let test_json = replace_heapless_string(&test_json, "\n", "").unwrap();
    let test_json = replace_heapless_string(&test_json, "   ", "").unwrap();

    debugcon_println!("{}", test_json);
}

/// Writes a JSON object indicating the failure of a test case, including its name, location, and failure message.
pub fn write_test_failure(test_name: &str, location: &str, message: &str) {
    let location = replace_heapless_string(&String::<MAX_STRING_LENGTH>::try_from(location).unwrap(), "\\", "/").unwrap(); // prevents escape issues with heapless String

    let test_json: String<MAX_STRING_LENGTH> = format!(r#"
{{
    "test": "{}",
    "result": "fail",
    "cycle_count": 0,
    "location": "{}",
    "message": "{}"
}}"#, test_name, location, message).unwrap();
    let test_json = replace_heapless_string(&test_json, "\n", "").unwrap();
    let test_json = replace_heapless_string(&test_json, "   ", "").unwrap();

    debugcon_println!("{}", test_json);
}

/// Helper function to replace all occurrences of a substring in a heapless String
fn replace_heapless_string(
    original: &String<MAX_STRING_LENGTH>,
    from: &str,
    to: &str,
) -> Result<String<MAX_STRING_LENGTH>, heapless::string::FromUtf16Error> {
    let mut new_string: String<MAX_STRING_LENGTH> = String::new();
    let mut last_end = 0;

    for (start, _) in original.match_indices(from) {
        new_string.push_str(&original[last_end..start])?;
        new_string.push_str(to)?;
        last_end = start + from.len();
    }
    new_string.push_str(&original[last_end..])?;
    Ok(new_string)
}
