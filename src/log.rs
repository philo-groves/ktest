use conquer_once::spin::OnceCell;
use heapless::String;
use spin::Mutex;
use uart_16550::SerialPort;
use crate::MAX_STRING_LENGTH;

/// The global serial port instance
pub static SERIAL1: OnceCell<Mutex<SerialPort>> = OnceCell::uninit();

/// Initialize the global serial port
fn init_serial() -> Mutex<SerialPort> {
    let mut serial_port = unsafe { SerialPort::new(0x3F8) };
    serial_port.init();
    Mutex::new(serial_port)
}

/// Print to the global serial port
#[doc(hidden)]
pub fn _serial_print(args: core::fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        let serial = SERIAL1.get_or_init(|| init_serial());
        serial.lock()
            .write_fmt(args)
            .expect("Printing to serial failed");
    });
}

/// Print to the global serial port
#[macro_export]
macro_rules! serial_print {
    ($($arg:tt)*) => {
        $crate::log::_serial_print(format_args!($($arg)*));
    };
}

/// Print to the global serial port, with a newline
#[macro_export]
macro_rules! serial_println {
    ($($arg:tt)*) => {
        $crate::log::_serial_print(format_args!($($arg)*));
        $crate::log::_serial_print(format_args!("\n"));
    };
}

/// Print to the debug console (macro helper)
#[doc(hidden)]
pub fn _debugcon_print(args: core::fmt::Arguments) {
    // convert args to string
    use core::fmt::Write;
    let mut s = String::<MAX_STRING_LENGTH>::new();
    s.write_fmt(args).expect("Failed to write to string");

    // this is unsafe because we are calling assembly code
    // in this case, writing to the debug console port (0xe9)
    // only a single byte at a time is written
    unsafe {
        for byte in s.bytes() {
            core::arch::asm!("out 0xe9, al", in("al") byte);
        }
    }
}

/// Print to the debug console
#[macro_export]
macro_rules! debugcon_print {
    ($($arg:tt)*) => {
        $crate::log::_debugcon_print(format_args!($($arg)*));
    };
}

/// Print to the debug console, with a newline
#[macro_export]
macro_rules! debugcon_println {
    ($($arg:tt)*) => {
        $crate::log::_debugcon_print(format_args!($($arg)*));
        $crate::log::_debugcon_print(format_args!("\n"));
    };
}
