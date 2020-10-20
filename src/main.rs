#![no_std]
#![no_main]

mod print;

use cortex_m_rt::entry;
use panic_halt as _;
use stm32f3xx_hal::{self as hal, pac, prelude::*};

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();
    let dp_c = cortex_m::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut delay = hal::delay::Delay::new(dp_c.SYST, clocks);

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

    let pins = (
        gpiob.pb6.into_af4(&mut gpiob.moder, &mut gpiob.afrl), // SCL
        gpiob.pb7.into_af4(&mut gpiob.moder, &mut gpiob.afrl), // SDA
    );
    let mut i2c = hal::i2c::I2c::i2c1(dp.I2C1, pins, 100.khz(), clocks, &mut rcc.apb1);

    println!("Start i2c scanning...");
    println!();
    delay.delay_ms(100_u32);

    for addr in 0x00_u8..=0x7F {
        match i2c.write(addr, &[]) {
            Ok(_) => print!("{:02x}", addr),
            _ => print!(".."),
        }
        if addr % 0x10 == 0x0F {
            println!();
        } else {
            print!(" ");
        }
        led.toggle().unwrap();
        delay.delay_ms(100_u32);
    }

    println!();
    println!("Done!");

    loop {}
}
