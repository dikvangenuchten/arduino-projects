#[cfg(target_arch = "avr")]
static mut LOGGER: Option<*mut dyn ufmt::uWrite<Error = core::convert::Infallible>> = None;

/// Registers a global serial logger used by `logln!`.
///
/// Safe here because firmware `main` is single-threaded and runs forever.
#[cfg(target_arch = "avr")]
pub fn init_logger<W>(writer: &mut W)
where
    W: ufmt::uWrite<Error = core::convert::Infallible>,
{
    unsafe {
        LOGGER = Some(writer as *mut W as *mut dyn ufmt::uWrite<Error = core::convert::Infallible>);
    }
}

#[cfg(not(target_arch = "avr"))]
pub fn init_logger<W>(_writer: &mut W) {}

#[cfg(target_arch = "avr")]
pub fn with_logger<F>(f: F)
where
    F: FnOnce(&mut dyn ufmt::uWrite<Error = core::convert::Infallible>),
{
    unsafe {
        if let Some(ptr) = LOGGER {
            f(&mut *ptr);
        }
    }
}

#[macro_export]
macro_rules! logln {
    ($fmt:literal $(, $arg:expr)* $(,)?) => {{
        #[cfg(target_arch = "avr")]
        {
            $crate::log::with_logger(|writer| {
                let _ = ufmt::uwriteln!(writer, $fmt $(, $arg)*);
            });
        }
        #[cfg(not(target_arch = "avr"))]
        {
            $(let _ = &$arg;)*
        }
    }};
}
