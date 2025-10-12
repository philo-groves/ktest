/// Exit QEMU with the given exit code. This function will not return.
/// 
/// The exit code is written to the I/O port `0xf4`, which is monitored
/// by QEMU to exit with the given code.
pub fn exit(exit_code: ExitCode) -> ! {
    use x86_64::instructions::{nop, port::Port};

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    loop {
        nop();
    }
}

/// Exit codes for QEMU. These codes are written to the I/O port `0xf4`
/// to signal QEMU to exit with the given code.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum ExitCode {
    Success = 0x10,
    _Failed = 0x11
}