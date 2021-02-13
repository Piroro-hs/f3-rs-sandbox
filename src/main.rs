#![no_std]
#![no_main]

mod print;

use core::sync::atomic::{AtomicUsize, Ordering};

use cortex_m::{asm, peripheral::NVIC};
use cortex_m_rt::entry;
use panic_halt as _;
use stm32f3xx_hal::{self as hal, pac, prelude::*};
use pac::interrupt;

#[derive(Clone, Copy, Debug)]
enum I2cLog {
    SclRise,
    SclFall,
    SdaRise,
    SdaFall,
}

static mut I2C_LOGS: [Option<I2cLog>; 256] = [None; 256];
static I2C_LOGS_CNT: AtomicUsize = AtomicUsize::new(0);

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let mut rcc = dp.RCC.constrain();
    let clocks = rcc
        .cfgr
        .use_hse(8.mhz())
        .bypass_hse()
        .sysclk(72.mhz())
        .hclk(72.mhz())
        .pclk1(36.mhz())
        .pclk2(72.mhz())
        .freeze(&mut flash.acr);

    let mut gpioa = dp.GPIOA.split(&mut rcc.ahb);
    let mut gpiob = dp.GPIOB.split(&mut rcc.ahb);

    let pins = (
        gpioa.pa2.into_af7_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrl), // TX
        gpioa.pa15.into_af7_push_pull(&mut gpioa.moder, &mut gpioa.otyper, &mut gpioa.afrh), // RX
    );
    let serial = hal::serial::Serial::usart2(dp.USART2, pins, 1_000_000.bps(), clocks, &mut rcc.apb1);
    let (tx, _) = serial.split();

    print::init(tx);

    let mut pins = (
        gpiob.pb6.into_af4_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl), // SCL
        gpiob.pb7.into_af4_open_drain(&mut gpiob.moder, &mut gpiob.otyper, &mut gpiob.afrl), // SDA
    );
    pins.0.internal_pull_up(&mut gpiob.pupdr, true);
    pins.1.internal_pull_up(&mut gpiob.pupdr, true);
    // let mut i2c = hal::i2c::I2c::new(dp.I2C1, pins, 100.khz(), clocks, &mut rcc.apb1); // missing logs
    let mut i2c = hal::i2c::I2c::new(dp.I2C1, pins, 1600.hz(), clocks, &mut rcc.apb1);

    unsafe { (*pac::RCC::ptr()).apb2enr.modify(|_, w| w.syscfgen().set_bit()) };
    dp.SYSCFG.exticr2.modify(|_, w| w.exti6().pb6().exti7().pb7());
    dp.EXTI.imr1.modify(|_, w| w.mr6().unmasked().mr7().unmasked());
    dp.EXTI.rtsr1.modify(|_, w| w.tr6().enabled().tr7().enabled());
    dp.EXTI.ftsr1.modify(|_, w| w.tr6().enabled().tr7().enabled());
    unsafe { NVIC::unmask(pac::Interrupt::EXTI9_5) };

    println!("start");

    let mut buf = [0_u8; 4];
    i2c.write_read(0x28, &[0x00], &mut buf).ok();
    println!("{:02X?}", buf);

    let mut scl = true;
    let mut sda = true;
    let mut bits = 0_u8;
    let mut bits_cnt = 0;
    for i in 0..I2C_LOGS_CNT.load(Ordering::Acquire) {
        let log = unsafe { I2C_LOGS[i] }.unwrap();
        println!("{:?}", log);
        match log {
            I2cLog::SclRise => {
                scl = true;
                println!("{:1}", sda as u8);
                if bits_cnt == 8 {
                    println!("{:02X}, ACK {}", bits, !sda);
                    bits = 0;
                    bits_cnt = 0;
                } else {
                    bits |= (sda as u8) << 7 - bits_cnt;
                    bits_cnt += 1;
                }
            },
            I2cLog::SclFall => {
                scl = false;
            },
            I2cLog::SdaRise => {
                sda = true;
                if scl {
                    println!("P");
                }
            },
            I2cLog::SdaFall => {
                sda = false;
                if scl {
                    println!("S");
                    bits = 0;
                    bits_cnt = 0;
                }
            },
        }
    }

    loop {
        asm::wfi();
    }
}

#[interrupt]
fn EXTI9_5() {
    let gpiob = pac::GPIOB::ptr();
    let exti = pac::EXTI::ptr();
    let cnt = I2C_LOGS_CNT.fetch_add(1, Ordering::SeqCst);
    unsafe {
        if (*exti).pr1.read().pr6().is_pending() {
            (*exti).pr1.write(|w| w.pr6().clear());
            I2C_LOGS[cnt] = Some(match (*gpiob).idr.read().idr6().variant() {
                pac::gpiob::idr::IDR15_A::HIGH => I2cLog::SclRise,
                pac::gpiob::idr::IDR15_A::LOW => I2cLog::SclFall,
            });
        } else if (*exti).pr1.read().pr7().is_pending() {
            (*exti).pr1.write(|w| w.pr7().clear());
            I2C_LOGS[cnt] = Some(match (*gpiob).idr.read().idr7().variant() {
                pac::gpiob::idr::IDR15_A::HIGH => I2cLog::SdaRise,
                pac::gpiob::idr::IDR15_A::LOW => I2cLog::SdaFall,
            });
        }
    }
}
