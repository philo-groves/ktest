# ktest

A custom test framework with features that are relevant to Rust-based operating system kernels.

**x86_64 is the only currently supported architecture**

## Examples

Single Crate: https://github.com/philo-groves/example-kernel-kboot-ktest

Workspace: https://github.com/philo-groves/example-kernel-kboot-ktest-multicrate

## Features:
- Custom `#[ktest]` macro for test functions
- Support for `#[ignore]` and `#[should_panic]` tags
- Custom `klib!("test_group");` macro for test setup:
  - Kernel entrypoint for tests
  - Panic handler for tests
  - Allows for function calling before/after tests
  - Allows for bootloader info injection
- Exports JSON data through QEMU `-debugcon` device
- Writes human-readable results through serial (CLI)
- Panic recovery; panic = current test failure
- Details for failure, e.g. line number and panic message
- Optionally link a basic heap allocator for tests (feature: `allocator`)

## Requirements
- A Rust-based kernel

**An allocator is NOT required for this library to function correctly. This library uses heapless structures without dynamic allocation.**

## Setup

The main.rs test setup is slightly more complex than lib.rs tests. In your main.rs, add the following `#![cfg_attr(test, ...)]` and `#[cfg(test)]` attributes/sections:

```
#![cfg_attr(test, feature(custom_test_frameworks))]            // enable custom test frameworks
#![cfg_attr(test, test_runner(ktest::runner))]                 // assign ktest as the runner
#![cfg_attr(test, reexport_test_harness_main = "test_main")]   // always use "test_main", expected by ktest/kboot

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
    // initialize the kernel

    // IMPORTANT: run tests if we are in test mode; this is the primary inclusion for main.rs
    #[cfg(test)]
    {
        ktest::init_harness("binary");
        test_main(); // should match reexport_test_harness_main
    }

    // continue running the kernel
}
```

#### Basic lib.rs example:

For lib.rs packages, there exists as `klib!("test group");` macro which will inject the proper source code for:

- Bootloader configuration
- Rust entrypoint
- Panic handling and recovery (panic = current test fail)
- A basic allocator, if enabled through the `allocator` feature (default: false)

See the relatively small source file: https://github.com/philo-groves/ktest/blob/main/src/macros/klib.rs

```
// the following 3 lines are the exact same from the main.rs example; both files require the setup
#![cfg_attr(test, feature(custom_test_frameworks))]            // enable custom test frameworks
#![cfg_attr(test, test_runner(ktest::runner))]                 // assign ktest as the runner
#![cfg_attr(test, reexport_test_harness_main = "test_main")]   // always use "test_main", expected by ktest/kboot

// "basic_crate" will act as a label for this test group; make sure this is test-only
#[cfg(test)]
ktest::klib!("basic_crate");
```

#### Complex librs. example:

The `klib!("test group");` macro can be expanded with two optional arguments: `klib_config` and `boot_config`:

- `klib_config`: Configurations for the test runner; currently, this holds function references to run before or after tests.
- `boot_config`: A direct reference to the [bootloader](https://github.com/rust-osdev/bootloader/blob/main/api/src/config.rs#L11) configuration

```
#[cfg(test)] // klib_config and boot_config are optional
ktest::klib!("extended_crate", klib_config = &KLIB_CONFIG, boot_config = &BOOTLOADER_CONFIG);

#[cfg(test)] // this config is optional
pub const KLIB_CONFIG: ktest::KlibConfig = ktest::KlibConfigBuilder::new_default()
    .before_tests(|boot_info| init(boot_info))
    .after_tests(|| teardown())
    .build();

#[cfg(test)] // this config is optional
pub const BOOTLOADER_CONFIG: bootloader_api::BootloaderConfig = {
    const HIGHER_HALF_START: u64 = 0xffff_8000_0000_0000;
    const PHYSICAL_MEMORY_OFFSET: u64 = 0x0000_0880_0000_0000;

    let mut config = bootloader_api::BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(bootloader_api::config::Mapping::FixedAddress(PHYSICAL_MEMORY_OFFSET));
    config.mappings.dynamic_range_start = Some(HIGHER_HALF_START);
    config
};

#[cfg(test)] // this function is optional
pub fn init(_boot_info: &'static bootloader_api::BootInfo) {
    // Setup code to run before tests, e.g. kernel initialization
}

#[cfg(test)] // this function is optional
pub fn teardown() {
    // Teardown code to run after tests
}
```

## Limine Support

`kboot` and `ktest` both support Limine as an optional bootloader. Both of these programs must have their `limine` feature enabled.

In your `kboot` runner, usually in .cargo/config.toml, use the `--limine` flag:
```
kboot --limine
```

In your kernel's `ktest` dependency inclusion, enable the `limine` feature:
```
[dev-dependencies]
ktest = { version = "0.1.6", features = ["limine"] }
```

## Example Output
```
################################################################
# Running 2 library tests for module: kernel::tests
----------------------------------------------------------------
lib_assertion                                               [pass]
lib_assertion_2                                             [fail] @ src\lib.rs:119: Make sure tests fail correctly
```

### Debugcon (JSON):

If you are using this library WITHOUT `kboot`, your JSON output will be line-delimited and look like this:

```
{"test_count":2,"test_group":"library"}
{"tests":["kernel::tests::lib_assertion","kernel::tests::lib_assertion_2"]}
{"cycle_count":866,"result":"pass","test":"kernel::tests::lib_assertion"}
{"cycle_count":0,"location":"src\\lib.rs:119","message":"Make sure tests fail correctly","result":"fail","test":"kernel::tests::lib_assertion_2"}
```

If you are using this library WITH `kboot`, the tool will reformat your line-delimited JSON output automatically and it will look like this:

```
{
  "test_group": "library",
  "summary": {
    "total": 2,
    "passed": 1,
    "failed": 1,
    "ignore": 0,
    "duration": 6266
  },
  "modules": [
    {
      "module": "kernel::tests",
      "tests": [
        {
          "test": "lib_assertion",
          "result": "pass",
          "cycle_count": 1142
        },
        {
          "test": "lib_assertion_2",
          "result": "fail",
          "cycle_count": 0,
          "location": "src\\lib.rs:119",
          "message": "Make sure tests fail correctly"
        }
      ]
    }
  ]
}
```
