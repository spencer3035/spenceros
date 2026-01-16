use conquer_once::spin::OnceCell;
use core::fmt::Write;

use crate::framebuffer::FrameBufferDisplay;
use crate::{Mutex, MyBootInfo};

pub fn init_loggers(into: &mut MyBootInfo) {
    let fb = into.framebuffer.take().unwrap();
    FRAMEBUFFER_WRITER.init_once(move || Mutex::new(fb));
    let port = into.port.take().unwrap();
    PORT_WRITER.init_once(move || Mutex::new(port));
}

#[macro_export]
macro_rules! print {
    ($($tt:tt)*) => {
        write!($crate::loggers::DynamicLogger, $($tt)*).unwrap()
    };
}
pub use print;

#[macro_export]
macro_rules! println {
    ($($tt:tt)*) => {
        $crate::loggers::print!($($tt)*);
        $crate::loggers::print!("\n");
    };
}
pub use println;

#[macro_export]
macro_rules! print_port {
    ($($tt:tt)*) => {
        write!($crate::loggers::PortLogger, $($tt)*).unwrap()
    };
}
pub use print_port;

#[macro_export]
macro_rules! println_port {
    ($($tt:tt)*) => {
        $crate::loggers::print_port!($($tt)*);
        $crate::loggers::print_port!("\n");
    };
}
pub use println_port;

#[macro_export]
macro_rules! print_framebuffer {
    ($($tt:tt)*) => {
        write!($crate::loggers::FrameBufferDisplay, $($tt)*).unwrap()
    };
}
pub use print_framebuffer;

#[macro_export]
macro_rules! println_framebuffer {
    ($($tt:tt)*) => {
        $crate::loggers::print_framebuffer!($($tt)*);
        $crate::loggers::print_framebuffer!("\n");
    };
}
pub use println_framebuffer;

static FRAMEBUFFER_WRITER: OnceCell<Mutex<FrameBufferDisplay>> = OnceCell::uninit();

static PORT_WRITER: OnceCell<Mutex<uart_16550::SerialPort>> = OnceCell::uninit();

pub(crate) struct DynamicLogger;

impl Write for DynamicLogger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        write!(FrameBufferLogger, "{s}").or_else(|_| write!(PortLogger, "{s}"))
    }
}

pub(crate) struct PortLogger;

impl Write for PortLogger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if let Some(mut port) = PORT_WRITER.get().map(|mutex| mutex.lock()) {
            write!(port, "{s}")
        } else {
            Err(core::fmt::Error)
        }
    }
}

pub(crate) struct FrameBufferLogger;

impl Write for FrameBufferLogger {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        if let Some(mut fb) = FRAMEBUFFER_WRITER.get().map(|mutex| mutex.lock()) {
            write!(fb, "{s}")
        } else {
            Err(core::fmt::Error)
        }
    }
}
