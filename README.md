# ktest

A custom test framework with features that are relevant to Rust-based operating system kernels.

**x86_64 is the only currently supported architecture**

## Features:
- Serial printer for pretty test output
- Panic handler to continue testing after panic (panic = current test fail)
- Details for failure, such as line number and panic message
- Tests sorted and executed by module
- JSON test output from QEMU using the `-debugcon` device

## Requirements
- A Rust-based kernel
- A functional allocator in your kernel (alloc is used by this library)

## Setup

Add/change the following to your main.rs:

```
// IMPORTANT: include `ktest::runner` as the test runner

#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, test_runner(ktest::runner))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]

fn kernel_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! { // any bootloader should work
    // initialize the kernel

    // IMPORTANT: run tests if we are in test mode
    #[cfg(test)]
    {
        ktest::init_harness("binary");
        test_main(); // should match reexport_test_harness_main
    }

    // continue running the kernel
}
```

And in your lib.rs:

```
// IMPORTANT: include `ktest::runner` as the test runner

#![cfg_attr(test, feature(custom_test_frameworks))]
#![cfg_attr(test, test_runner(ktest::runner))]
#![cfg_attr(test, reexport_test_harness_main = "test_main")]

// IMPORTANT: make sure your library has a test-only start function

#[cfg(test)]
bootloader_api::entry_point!(kernel_test_main, config = &BOOTLOADER_CONFIG); // any bootloader should work

#[cfg(test)]
fn kernel_test_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! { // any bootloader should work
    // initialize the kernel

    ktest::init_harness("library");
    test_main();
    
    // continue running the kernel
}

// IMPORTANT: You should have a panic handler specifically for tests

#[cfg(not(test))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    // your original panic handler
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    ktest::panic(info) // delegate to `ktest`
}
```

## Example Output

### Serial (pretty print):
```
################################################################
# Running 2 library tests for module: kernel::allocator::tests
----------------------------------------------------------------
allocator_assertion_1                                       [ok]
allocator_assertion_2                                       [fail] @ src\allocator.rs:49: Make sure tests fail correctly

################################################################
# Running 2 library tests for module: kernel::memory::tests
----------------------------------------------------------------
memory_assertion_1                                          [ok]
memory_assertion_2                                          [ok]

################################################################
# Running 2 library tests for module: kernel::tests
----------------------------------------------------------------
lib_assertion                                               [ok]
lib_assertion_2                                             [fail] @ src\lib.rs:119: Make sure tests fail correctly
```

### Debugcon (JSON):

If you are using this library WITHOUT `kboot`, your JSON output will be line-delimited and look like this:

```
{"test_count":6,"test_group":"library"}
{"tests":["kernel::allocator::tests::allocator_assertion_1","kernel::allocator::tests::allocator_assertion_2","kernel::memory::tests::memory_assertion_1","kernel::memory::tests::memory_assertion_2","kernel::tests::lib_assertion","kernel::tests::lib_assertion_2"]}
{"cycle_count":1242,"result":"ok","test":"kernel::allocator::tests::allocator_assertion_1"}
{"cycle_count":0,"location":"src\\allocator.rs:49","message":"Make sure tests fail correctly","result":"fail","test":"kernel::allocator::tests::allocator_assertion_2"}
{"cycle_count":1226,"result":"ok","test":"kernel::memory::tests::memory_assertion_1"}
{"cycle_count":726,"result":"ok","test":"kernel::memory::tests::memory_assertion_2"}
{"cycle_count":866,"result":"ok","test":"kernel::tests::lib_assertion"}
{"cycle_count":0,"location":"src\\lib.rs:119","message":"Make sure tests fail correctly","result":"fail","test":"kernel::tests::lib_assertion_2"}
```

If you are using this library WITH `kboot`, the tool will reformat your line-delimited JSON output automatically and it will look like this:

```
{
  "test_group": "library",
  "summary": {
    "total": 6,
    "passed": 4,
    "failed": 2,
    "missed": 0,
    "duration": 6266
  },
  "modules": [
    {
      "module": "kernel::allocator::tests",
      "tests": [
        {
          "test": "allocator_assertion_1",
          "result": "ok",
          "cycle_count": 1660
        },
        {
          "test": "allocator_assertion_2",
          "result": "fail",
          "cycle_count": 0,
          "location": "src\\allocator.rs:49",
          "message": "Make sure tests fail correctly"
        }
      ]
    },
    {
      "module": "kernel::memory::tests",
      "tests": [
        {
          "test": "memory_assertion_1",
          "result": "ok",
          "cycle_count": 776
        },
        {
          "test": "memory_assertion_2",
          "result": "ok",
          "cycle_count": 1050
        }
      ]
    },
    {
      "module": "kernel::tests",
      "tests": [
        {
          "test": "lib_assertion",
          "result": "ok",
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