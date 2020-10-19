use core::cell::RefCell;
use core::convert::Infallible;
use core::fmt::{Arguments, Write};

use cortex_m::interrupt::{free, Mutex};
use stm32f3xx_hal::{hal::serial::Write as SerialWrite, pac::USART2, serial::Tx};

static TX: Mutex<RefCell<Option<Tx<USART2>>>> = Mutex::new(RefCell::new(None));

pub fn init(tx: Tx<USART2>) {
    free(|cs| {
        TX.borrow(cs).replace(Some(tx));
    });
}

#[doc(hidden)]
pub fn _print(args: Arguments) {
    free(|cs| {
        if let Some(tx) = &mut *TX.borrow(cs).borrow_mut() {
            (tx as &mut dyn SerialWrite<u8, Error = Infallible>).write_fmt(args).ok();
        }
    })
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::print::_print(format_args!($($arg)*))) 
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}
