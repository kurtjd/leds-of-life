#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use {defmt_rtt as _, panic_probe as _};

use stm32f1xx_hal::{pac::Peripherals, prelude::*};

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let mut gpioc = dp.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut delay = cp.SYST.delay(&clocks);

    println!("BLINKY");
    loop {
        led.set_high();
        delay.delay_ms(100u32);
        led.set_low();
        delay.delay_ms(100u32);

        println!("BLINK");
    }
}
