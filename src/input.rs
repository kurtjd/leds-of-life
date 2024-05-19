use stm32f1xx_hal::{
    adc::Adc,
    gpio::{self, Analog, Input, PullUp},
    pac::ADC1,
    prelude::*,
};

use crate::game::*;

const ROWS: usize = 16;
const COLS: usize = 16;

pub struct Buttons {
    up: gpio::PA8<Input<PullUp>>,
    down: gpio::PA11<Input<PullUp>>,
    left: gpio::PA9<Input<PullUp>>,
    right: gpio::PA12<Input<PullUp>>,
    select: gpio::PA10<Input<PullUp>>,
    pause: gpio::PA15<Input<PullUp>>,
    pub pressed: bool,
}

impl Buttons {
    pub fn handle(&mut self, life: &mut Life, state: &mut State) {
        if state.paused {
            if self.up.is_low() {
                if state.xhair_row > 0 {
                    state.xhair_row -= 1;
                } else {
                    state.xhair_row = ROWS - 1;
                }
            }
            if self.down.is_low() {
                if state.xhair_row < ROWS - 1 {
                    state.xhair_row += 1;
                } else {
                    state.xhair_row = 0;
                }
            }
            if self.left.is_low() {
                if state.xhair_col > 0 {
                    state.xhair_col -= 1;
                } else {
                    state.xhair_col = COLS - 1;
                }
            }
            if self.right.is_low() {
                if state.xhair_col < COLS - 1 {
                    state.xhair_col += 1;
                } else {
                    state.xhair_col = 0;
                }
            }
            if self.select.is_low() {
                if life.get_cell(state.xhair_col, state.xhair_row) == 1 {
                    life.cells[state.xhair_row] &= !(1 << (COLS - state.xhair_col - 1));
                } else {
                    life.cells[state.xhair_row] |= 1 << (COLS - state.xhair_col - 1);
                }
            }

            if self.up.is_low()
                || self.down.is_low()
                || self.left.is_low()
                || self.right.is_low()
                || self.select.is_low()
            {
                self.pressed = true;
                state.xhair_moved = true;
            }
        }

        if self.pause.is_low() {
            state.paused = !state.paused;

            self.pressed = true;
            state.xhair_moved = true;
        }
    }

    pub fn new(
        up: gpio::PA8<Input<PullUp>>,
        down: gpio::PA11<Input<PullUp>>,
        left: gpio::PA9<Input<PullUp>>,
        right: gpio::PA12<Input<PullUp>>,
        select: gpio::PA10<Input<PullUp>>,
        pause: gpio::PA15<Input<PullUp>>,
    ) -> Self {
        Self {
            up,
            down,
            left,
            right,
            select,
            pause,
            pressed: false,
        }
    }
}

pub struct Pots {
    delay_pot: gpio::PA0<Analog>,
    brightness_pot: gpio::PA1<Analog>,
    adc: Adc<ADC1>,
}

impl Pots {
    pub fn read(&mut self) -> (u16, u8) {
        let delay_ms: u16 = self.adc.read(&mut self.delay_pot).unwrap();
        let brightness: u16 = self.adc.read(&mut self.brightness_pot).unwrap();

        // Not a whole lot of thought put into these divisors, just trial and error
        (delay_ms / 10, (brightness / 256) as u8)
    }

    pub fn new(
        delay_pot: gpio::PA0<Analog>,
        brightness_pot: gpio::PA1<Analog>,
        adc: Adc<ADC1>,
    ) -> Self {
        Self {
            delay_pot,
            brightness_pot,
            adc,
        }
    }
}
