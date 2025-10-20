use conquer_once::spin::OnceCell;
use heapless::String;
use crate::MAX_STRING_LENGTH;

/// A global variable to hold the test group name (only one test group per binary)
static TEST_GROUP: OnceCell<String<MAX_STRING_LENGTH>> = OnceCell::uninit();

/// Sets the test group name. This should be called once during test initialization.
pub fn set_test_group(name: &str) {
    TEST_GROUP.get_or_init(|| name.try_into().unwrap());
}

/// Gets the test group name, if set.
pub fn get_test_group() -> Option<&'static str> {
    TEST_GROUP.get().map(|s| s.as_str())
}
