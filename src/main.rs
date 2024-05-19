#![no_std]
#![no_main]

mod game;
mod input;
mod matrix;

//use defmt::*;
use cortex_m_rt::entry;
use stm32f1xx_hal::{
    adc,
    pac::Peripherals,
    prelude::*,
    spi::{NoMiso, Spi},
};
use {defmt_rtt as _, panic_probe as _};

use game::*;
use input::*;
use matrix::*;

#[entry]
fn main() -> ! {
    // Peripherals
    let dp = Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    // Clocks and timers
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    // Max SPEED!
    let clocks = rcc
        .cfgr
        .use_hse(8.MHz())
        .sysclk(72.MHz())
        .pclk1(36.MHz())
        .pclk2(72.MHz())
        .freeze(&mut flash.acr);
    let mut delay = cp.SYST.delay(&clocks);

    // GPIO
    let mut afio = dp.AFIO.constrain();
    let mut gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split(); // Necessary to pass into disable_jtag

    // Buttons
    let up_btn = gpioa.pa8.into_pull_up_input(&mut gpioa.crh);
    let down_btn = gpioa.pa11.into_pull_up_input(&mut gpioa.crh);
    let left_btn = gpioa.pa9.into_pull_up_input(&mut gpioa.crh);
    let right_btn = gpioa.pa12.into_pull_up_input(&mut gpioa.crh);
    let sel_btn = gpioa.pa10.into_pull_up_input(&mut gpioa.crh);

    /* PA15 is initially reserved for JTAG use, but since we are not using it, disable it.
     * We don't need PB3 or PB4 though so throw them away.
     */
    let (pause_btn, _, _) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);
    let pause_btn = pause_btn.into_pull_up_input(&mut gpioa.crh);

    // Potentiometers
    let adc = adc::Adc::adc1(dp.ADC1, clocks);
    let delay_pot = gpioa.pa0.into_analog(&mut gpioa.crl);
    let brightness_pot = gpioa.pa1.into_analog(&mut gpioa.crl);

    // SPI
    let cs = gpioa.pa4.into_push_pull_output(&mut gpioa.crl);
    let sck = gpioa.pa5.into_alternate_push_pull(&mut gpioa.crl);
    let mosi = gpioa.pa7.into_alternate_push_pull(&mut gpioa.crl);
    let spi = Spi::spi1(
        dp.SPI1,
        (sck, NoMiso, mosi),
        &mut afio.mapr,
        MATRIX_SPI_MODE,
        1.MHz(),
        clocks,
    );

    // Consume everything and get back nice structs :)
    let mut buttons = Buttons::new(up_btn, down_btn, left_btn, right_btn, sel_btn, pause_btn);
    let mut pots = Pots::new(delay_pot, brightness_pot, adc);
    let mut matrix = Matrix::new(spi, cs);
    let mut state = State::new();
    let mut life = Life::new();

    // Start off with the classic glider, symbol of hackers everywhere!
    life.draw_glider();

    loop {
        /* Very simple (and slightly error prone) polling method of input handling.
         * There are obviously better ways of doing this, but this is a quick project
         * so this is good enough.
         */
        buttons.handle(&mut life, &mut state);
        if buttons.pressed {
            delay.delay_ms(150u32);
            buttons.pressed = false;
        }

        // Read the potentiometers and adjust brightness and delay accordingly
        let (delay_ms, brightness) = pots.read();
        matrix.update_brightness(brightness);

        /* If we are paused, only update the matrix if the crosshair has been moved.
         * If not paused, update the matrix and generate the next cycle of life.
         */
        if state.paused && state.xhair_moved {
            matrix.update_leds(&life);
            matrix.draw_xhair(&life, &state);
            state.xhair_moved = false;
        } else if !state.paused {
            matrix.update_leds(&life);
            life.live();
            delay.delay_ms(delay_ms);
        }
    }
}
