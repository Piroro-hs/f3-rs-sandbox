#![no_std]
#![no_main]

mod print;

use cortex_m::asm;
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f3xx_hal::{self as hal, pac, prelude::*};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);
    
    let mut led = gpiob.pb3.into_push_pull_output(&mut gpiob.moder, &mut gpiob.otyper);

    let pins = (
        gpioa.pa2.into_af7(&mut gpioa.moder, &mut gpioa.afrl), // TX
        gpioa.pa15.into_af7(&mut gpioa.moder, &mut gpioa.afrh), // RX
    );
    let serial = hal::serial::Serial::usart2(dp.USART2, pins, 9600.bps(), clocks, &mut rcc.apb1);
    let (tx, _) = serial.split();

    print::init(tx);

    println!("start");
    
    loop {
        led.toggle().unwrap();
        asm::delay(8_000_000);
    }
}
