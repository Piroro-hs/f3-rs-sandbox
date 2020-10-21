#![no_std]
#![no_main]

use cortex_m::{asm, peripheral::syst::SystClkSource};
use cortex_m_rt::{entry, exception};
use panic_halt as _;
use stm32f3xx_hal::{self as hal, pac, prelude::*};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let mut dp_c = cortex_m::Peripherals::take().unwrap();

    let mut rcc = dp.RCC.constrain();
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);

    let mut led = gpiob.pb3.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    dp_c.SYST.set_clock_source(SystClkSource::External);
    dp_c.SYST.set_reload(500_000);
    dp_c.SYST.clear_current();
    dp_c.SYST.enable_interrupt();
    dp_c.SYST.enable_counter();

    loop {
        led.toggle().unwrap();
        asm::wfi();
    }
}

#[exception]
fn SysTick() {}
