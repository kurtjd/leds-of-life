#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use max7219::*;
use {defmt_rtt as _, panic_probe as _};

use stm32f1xx_hal::{
    adc,
    pac::Peripherals,
    prelude::*,
    spi::{Mode, NoMiso, Phase, Polarity, Spi},
};

// Figure out how import this from HAL
pub const MODE_0: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

#[entry]
fn main() -> ! {
    let dp = Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut delay = cp.SYST.delay(&clocks);

    let mut adc = adc::Adc::adc1(dp.ADC1, clocks);

    let mut afio = dp.AFIO.constrain();
    let mut gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();

    let up_btn = gpioa.pa8.into_pull_up_input(&mut gpioa.crh);
    let down_btn = gpioa.pa11.into_pull_up_input(&mut gpioa.crh);
    let left_btn = gpioa.pa9.into_pull_up_input(&mut gpioa.crh);
    let right_btn = gpioa.pa12.into_pull_up_input(&mut gpioa.crh);

    let sel_btn = gpioa.pa10.into_pull_up_input(&mut gpioa.crh);

    let (pause_btn, _, _) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);
    let pause_btn = pause_btn.into_pull_up_input(&mut gpioa.crh);

    let mut speed_pot = gpioa.pa0.into_analog(&mut gpioa.crl);
    let mut brightness_pot = gpioa.pa1.into_analog(&mut gpioa.crl);

    let cs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);
    let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
    let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);

    let spi = Spi::spi1(
        dp.SPI1,
        (sck, NoMiso, mosi),
        &mut afio.mapr,
        MODE_0,
        1.MHz(),
        clocks,
    );

    let mut matrix = MAX7219::from_spi_cs(4, spi, cs).unwrap();
    matrix.power_on().unwrap();
    println!("Matrix powered on");

    for i in 0..4 {
        println!("Start idx {} test...", i);
        matrix.test(i, true).unwrap();
        delay.delay_ms(1000u32);
    }

    loop {
        let speed: u16 = adc.read(&mut speed_pot).unwrap();
        println!("Speed: {}", speed);

        let brightness: u16 = adc.read(&mut brightness_pot).unwrap();
        println!("Brightness: {}", brightness);

        println!("Up pressed: {}", up_btn.is_low());
        println!("Down pressed: {}", down_btn.is_low());
        println!("Left pressed: {}", left_btn.is_low());
        println!("Right pressed: {}", right_btn.is_low());

        println!("Sel pressed: {}", sel_btn.is_low());
        println!("Pause pressed: {}", pause_btn.is_low());

        delay.delay_ms(1000u32);
    }
}
