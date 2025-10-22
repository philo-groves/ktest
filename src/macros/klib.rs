/// `klib!` function-like macro
#[macro_export]
macro_rules! klib {
    // test group only
    ($test_group:literal) => {
        $crate::klib!($test_group, klib_config = &ktest::KlibConfig::new_default(), boot_config = &bootloader_api::BootloaderConfig::new_default());
    };

    // test group + klib config
    ($test_group:literal, klib_config = &$klib_config:expr) => {
        $crate::klib!($test_group, klib_config = &$klib_config, boot_config = &bootloader_api::BootloaderConfig::new_default());
    };

    // test group + boot config
    ($test_group:literal, boot_config = &$boot_config:expr) => {
        $crate::klib!($test_group, klib_config = &ktest::KlibConfig::new_default(), boot_config = &$boot_config);
    };

    // test group + klib config + boot config
    ($test_group:literal, klib_config = &$klib_config:expr, boot_config = &$boot_config:expr) => {
        #[cfg(test)] // it is important to only include this code in test builds
        const _: () = {
            // note: the triple underscore (___) prefix is to avoid name collisions

            static ___BOOTLOADER_CONFIG: bootloader_api::BootloaderConfig = $boot_config;
            static ___KLIB_CONFIG: ktest::KlibConfig = $klib_config;

            #[panic_handler]
            fn ___panic(info: &core::panic::PanicInfo) -> ! {
                ktest::panic(info)
            }

            fn ___kernel_test_main(boot_info: &'static mut bootloader_api::BootInfo) -> ! {
                ktest::init_harness($test_group);
                ktest::memory::heap::init_allocator_if_enabled(boot_info)
                    .expect("Heap allocator initialization failed");

                if let Some(before_tests) = ___KLIB_CONFIG.before_tests {
                    before_tests(boot_info);
                }

                test_main();

                if let Some(after_tests) = ___KLIB_CONFIG.after_tests {
                    after_tests();
                }

                loop {
                    // It may seem preferable to use the x86_64 `hlt` instruction here, but
                    // that would require any crates using this macro to depend on `x86_64`,
                    // which is not desirable. Using inline assembly avoids that dependency.
                    //
                    // note: this is the exact same instruction as `x86_64::instructions::hlt()`
                    unsafe { core::arch::asm!("hlt", options(nomem, nostack, preserves_flags)); }
                }
            }

            bootloader_api::entry_point!(___kernel_test_main, config = &___BOOTLOADER_CONFIG);
        };
    };
}

pub struct KlibConfig {
    pub before_tests: Option<fn(&'static bootloader_api::BootInfo)>,
    pub after_tests: Option<fn()>
}

impl KlibConfig {
    pub const fn new_default() -> Self {
        KlibConfig {
            before_tests: None,
            after_tests: None
        }
    }
}

pub struct KlibConfigBuilder {
    pub before_tests: Option<fn(&'static bootloader_api::BootInfo)>,
    pub after_tests: Option<fn()>
}

impl KlibConfigBuilder {
    pub const fn new_default() -> Self {
        KlibConfigBuilder {
            before_tests: None,
            after_tests: None
        }
    }

    pub const fn new(before_tests: Option<fn(&'static bootloader_api::BootInfo)>, after_tests: Option<fn()>) -> Self {
        KlibConfigBuilder {
            before_tests,
            after_tests
        }
    }

    pub const fn build(self) -> KlibConfig {
        KlibConfig {
            before_tests: self.before_tests,
            after_tests: self.after_tests
        }
    }

    pub const fn before_tests(mut self, before_tests: fn(&'static bootloader_api::BootInfo)) -> Self {
        self.before_tests = Some(before_tests);
        self
    }

    pub const fn after_tests(mut self, after_tests: fn()) -> Self {
        self.after_tests = Some(after_tests);
        self
    }
}
