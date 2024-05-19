#![no_std]
#![no_main]

use cortex_m_rt::entry;
use defmt::*;
use max7219::{connectors::SpiConnectorSW, *};
use {defmt_rtt as _, panic_probe as _};

use stm32f1xx_hal::{
    adc::{self, Adc},
    gpio::{self, Alternate, Analog, Input, Output, PullUp},
    pac::{Peripherals, ADC1, SPI1},
    prelude::*,
    spi::{Mode, NoMiso, Phase, Polarity, Spi, Spi1NoRemap},
};

const ROWS: u8 = 16;
const COLS: u8 = 16;

type Max7219Type = MAX7219<
    SpiConnectorSW<
        Spi<SPI1, Spi1NoRemap, (gpio::PA5<Alternate>, NoMiso, gpio::PA7<Alternate>), u8>,
        gpio::PA4<Output>,
    >,
>;

// Figure out how import this from HAL
pub const MODE_0: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

struct Buttons {
    up: gpio::PA8<Input<PullUp>>,
    down: gpio::PA11<Input<PullUp>>,
    left: gpio::PA9<Input<PullUp>>,
    right: gpio::PA12<Input<PullUp>>,
    select: gpio::PA10<Input<PullUp>>,
    pause: gpio::PA15<Input<PullUp>>,
    pressed: bool,
}

impl Buttons {
    fn new(
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

    fn handle(&mut self, life: &mut Life, state: &mut State) {
        if state.paused {
            if self.up.is_low() {
                if state.xhair_row > 0 {
                    state.xhair_row -= 1;
                } else {
                    state.xhair_row = ROWS - 1;
                }

                self.pressed = true;
                state.xhair_moved = true;
            }
            if self.down.is_low() {
                if state.xhair_row < ROWS - 1 {
                    state.xhair_row += 1;
                } else {
                    state.xhair_row = 0;
                }

                self.pressed = true;
                state.xhair_moved = true;
            }
            if self.left.is_low() {
                if state.xhair_col > 0 {
                    state.xhair_col -= 1;
                } else {
                    state.xhair_col = COLS - 1;
                }

                self.pressed = true;
                state.xhair_moved = true;
            }
            if self.right.is_low() {
                if state.xhair_col < COLS - 1 {
                    state.xhair_col += 1;
                } else {
                    state.xhair_col = 0;
                }

                self.pressed = true;
                state.xhair_moved = true;
            }

            if self.select.is_low() {
                if life.get_cell(state.xhair_col, state.xhair_row) == 1 {
                    life.cells[state.xhair_row as usize] &= !(1 << (COLS - state.xhair_col - 1));
                } else {
                    life.cells[state.xhair_row as usize] |= 1 << (COLS - state.xhair_col - 1);
                }

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
}

struct Pots {
    speed: gpio::PA0<Analog>,
    brightness: gpio::PA1<Analog>,
    adc: Adc<ADC1>,
}

impl Pots {
    fn new(speed: gpio::PA0<Analog>, brightness: gpio::PA1<Analog>, adc: Adc<ADC1>) -> Self {
        Self {
            speed,
            brightness,
            adc,
        }
    }

    fn read(&mut self) -> (u16, u8) {
        let speed: u16 = self.adc.read(&mut self.speed).unwrap();
        let brightness: u16 = self.adc.read(&mut self.brightness).unwrap();

        (speed / 2, (brightness / 256) as u8)
    }
}

struct State {
    paused: bool,
    xhair_moved: bool,
    xhair_row: u8,
    xhair_col: u8,
}

impl State {
    fn new() -> Self {
        Self {
            paused: true,
            xhair_moved: true,
            xhair_row: 7,
            xhair_col: 7,
        }
    }
}

struct Life {
    cells: [u16; ROWS as usize],
}

impl Life {
    fn get_cell(&self, x: u8, y: u8) -> u8 {
        ((self.cells[y as usize] >> (COLS - x - 1)) & 1) as u8
    }

    fn count_neighbors(&self, x: u8, y: u8) -> u8 {
        let mut total = 0;

        for ny in y as i32 - 1..=y as i32 + 1 {
            for nx in x as i32 - 1..=x as i32 + 1 {
                let tmp_y = if ny < 0 {
                    ROWS - 1
                } else if ny as u8 >= ROWS {
                    0
                } else {
                    ny as u8
                };

                let tmp_x = if nx < 0 {
                    COLS - 1
                } else if nx as u8 >= COLS {
                    0
                } else {
                    nx as u8
                };

                if tmp_y != y || tmp_x != x {
                    total += self.get_cell(tmp_x, tmp_y);
                }
            }
        }

        total
    }

    fn live(&mut self) {
        let mut new_cells = self.cells;

        for y in 0..ROWS {
            for x in 0..COLS {
                let neighbors = self.count_neighbors(x, y);

                if !(2..=3).contains(&neighbors) {
                    new_cells[y as usize] &= !(1 << (COLS - x - 1));
                } else if neighbors == 3 || (self.get_cell(x, y) == 1 && neighbors == 2) {
                    new_cells[y as usize] |= 1 << (COLS - x - 1);
                }
            }
        }

        self.cells = new_cells;
    }

    fn glider(&mut self) {
        self.cells[0] = 0x4000;
        self.cells[1] = 0x2000;
        self.cells[2] = 0xE000;
    }

    fn new() -> Self {
        Self {
            cells: [0; ROWS as usize],
        }
    }
}

struct Matrix {
    max7219: Max7219Type,
}

impl Matrix {
    fn update_brightness(&mut self, brightness: u8) {
        for i in 0..4 {
            self.max7219.set_intensity(i, brightness).unwrap();
        }
    }

    fn update_leds(&mut self, life: &Life) {
        let mut quad = [[0; 8]; 4];

        for i in 0..8 {
            quad[0][i] = (life.cells[i] >> 8) as u8;
            quad[1][i] = (life.cells[i] & 0xFF) as u8;
            quad[2][i] = (life.cells[i + 8] >> 8) as u8;
            quad[3][i] = (life.cells[i + 8] & 0xFF) as u8;
        }

        for (i, bytes) in quad.iter().enumerate() {
            self.max7219.write_raw(i, bytes).unwrap();
        }
    }

    fn set_column(&mut self, col: u8, skip_row: usize, life: &Life) {
        let mut cells = [[0; 8]; 2];

        for i in 0..8 {
            let bit1 = if i != skip_row {
                1 << (COLS as u16 - col as u16 - 1)
            } else {
                0
            };
            let bit2 = if (i + 8) != skip_row {
                1 << (COLS as u16 - col as u16 - 1)
            } else {
                0
            };

            if col <= 7 {
                cells[0][i] = ((life.cells[i] | bit1) >> 8) as u8;
                cells[1][i] = ((life.cells[i + 8] | bit2) >> 8) as u8;
            } else {
                cells[0][i] = ((life.cells[i] | bit1) & 0xFF) as u8;
                cells[1][i] = ((life.cells[i + 8] | bit2) & 0xFF) as u8;
            }
        }

        let quads = if col <= 7 { [0, 2] } else { [1, 3] };
        for (i, q) in quads.iter().enumerate() {
            self.max7219.write_raw(*q, &cells[i]).unwrap();
        }
    }

    fn set_row(&mut self, row: u8, skip_col: u8, _life: &Life) {
        let byte1 = if skip_col <= 7 {
            !(1 << (8 - skip_col - 1))
        } else {
            0xFF
        };
        let byte2 = if skip_col > 7 && skip_col < COLS {
            !(1 << (8 - (skip_col - 8) - 1))
        } else {
            0xFF
        };

        if row <= 7 {
            self.max7219.write_raw_byte(0, row + 1, byte1).unwrap();
            self.max7219.write_raw_byte(1, row + 1, byte2).unwrap();
        } else {
            self.max7219
                .write_raw_byte(2, (row - 8) + 1, byte1)
                .unwrap();
            self.max7219
                .write_raw_byte(3, (row - 8) + 1, byte2)
                .unwrap();
        }
    }

    fn draw_xhair(&mut self, life: &Life, state: &State) {
        let mut skip_row = ROWS;
        let mut skip_col = COLS;

        if life.get_cell(state.xhair_col, state.xhair_row) == 0 {
            skip_row = state.xhair_row;
            skip_col = state.xhair_col;
        }

        self.set_column(state.xhair_col, skip_row as usize, life);
        self.set_row(state.xhair_row, skip_col, life);
    }

    fn new(max7219: Max7219Type) -> Self {
        Self { max7219 }
    }
}

#[entry]
fn main() -> ! {
    // Peripherals
    let dp = Peripherals::take().unwrap();
    let cp = cortex_m::peripheral::Peripherals::take().unwrap();

    // Clocks and timers
    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze(&mut flash.acr);
    let mut delay = cp.SYST.delay(&clocks);

    // Input
    let mut afio = dp.AFIO.constrain();
    let mut gpioa = dp.GPIOA.split();
    let gpiob = dp.GPIOB.split();

    // Buttons
    let up_btn = gpioa.pa8.into_pull_up_input(&mut gpioa.crh);
    let down_btn = gpioa.pa11.into_pull_up_input(&mut gpioa.crh);
    let left_btn = gpioa.pa9.into_pull_up_input(&mut gpioa.crh);
    let right_btn = gpioa.pa12.into_pull_up_input(&mut gpioa.crh);
    let sel_btn = gpioa.pa10.into_pull_up_input(&mut gpioa.crh);
    let (pause_btn, _, _) = afio.mapr.disable_jtag(gpioa.pa15, gpiob.pb3, gpiob.pb4);
    let pause_btn = pause_btn.into_pull_up_input(&mut gpioa.crh);
    let mut buttons = Buttons::new(up_btn, down_btn, left_btn, right_btn, sel_btn, pause_btn);

    // Potentiometers
    let adc = adc::Adc::adc1(dp.ADC1, clocks);
    let speed_pot = gpioa.pa0.into_analog(&mut gpioa.crl);
    let brightness_pot = gpioa.pa1.into_analog(&mut gpioa.crl);
    let mut pots = Pots::new(speed_pot, brightness_pot, adc);

    // SPI
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

    // LED Matrix Driver
    let mut matrix = Matrix::new(MAX7219::from_spi_cs(4, spi, cs).unwrap());
    matrix.max7219.power_on().unwrap();
    println!("Matrix powered on");

    let mut state = State::new();
    let mut life = Life::new();
    life.glider();

    loop {
        buttons.handle(&mut life, &mut state);
        if buttons.pressed {
            delay.delay_ms(150u32);
            buttons.pressed = false;
        }

        let (speed, brightness) = pots.read();
        matrix.update_brightness(brightness);

        if state.paused && state.xhair_moved {
            matrix.update_leds(&life);
            matrix.draw_xhair(&life, &state);
            state.xhair_moved = false;
        } else if !state.paused {
            matrix.update_leds(&life);
            life.live();
            delay.delay_ms(speed);
        }
    }
}
