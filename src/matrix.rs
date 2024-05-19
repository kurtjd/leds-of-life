use stm32f1xx_hal::{
    gpio::{self, Alternate, Output},
    pac::SPI1,
    spi::{Mode, NoMiso, Phase, Polarity, Spi, Spi1NoRemap},
};

use crate::game::*;
use max7219::{connectors::SpiConnectorSW, *};

/* The number of sub 8x8 matrices.
 * They are arranged into 4 quadrants.
 */
const NUM_DISPLAYS: usize = 4;
const ROWS: usize = 16;
const HALF_ROWS: usize = ROWS / 2;
const COLS: usize = 16;
const HALF_COLS: usize = COLS / 2;

pub type MatrixSpi =
    Spi<SPI1, Spi1NoRemap, (gpio::PA5<Alternate>, NoMiso, gpio::PA7<Alternate>), u8>;
pub type MatrixCs = gpio::PA4<Output>;
pub const MATRIX_SPI_MODE: Mode = Mode {
    polarity: Polarity::IdleLow,
    phase: Phase::CaptureOnFirstTransition,
};

type Max7219Type = MAX7219<SpiConnectorSW<MatrixSpi, MatrixCs>>;

pub struct Matrix {
    max7219: Max7219Type,
}

/* I feel set_column and set_row are written too C-like, will experiment making these Rustier. */
impl Matrix {
    fn set_column(&mut self, col: usize, skip_row: usize, life: &Life) {
        let mut cells = [[0; HALF_ROWS]; 2];

        for i in 0..HALF_ROWS {
            let bit1 = if i != skip_row {
                1 << (COLS as u16 - col as u16 - 1)
            } else {
                0
            };
            let bit2 = if (i + HALF_ROWS) != skip_row {
                1 << (COLS as u16 - col as u16 - 1)
            } else {
                0
            };

            if col < HALF_COLS {
                cells[0][i] = ((life.cells[i] | bit1) >> 8) as u8;
                cells[1][i] = ((life.cells[i + HALF_ROWS] | bit2) >> 8) as u8;
            } else {
                cells[0][i] = ((life.cells[i] | bit1) & 0xFF) as u8;
                cells[1][i] = ((life.cells[i + HALF_ROWS] | bit2) & 0xFF) as u8;
            }
        }

        let quads = if col < HALF_COLS { [0, 2] } else { [1, 3] };
        for (i, q) in quads.iter().enumerate() {
            self.max7219.write_raw(*q, &cells[i]).unwrap();
        }
    }

    fn set_row(&mut self, row: usize, skip_col: usize) {
        let byte1 = if skip_col < HALF_COLS {
            !(1 << (HALF_COLS - skip_col - 1))
        } else {
            0xFF
        };
        let byte2 = if (HALF_COLS..COLS).contains(&skip_col) {
            !(1 << (HALF_COLS - (skip_col - HALF_COLS) - 1))
        } else {
            0xFF
        };

        if row < HALF_ROWS {
            self.max7219
                .write_raw_byte(0, row as u8 + 1, byte1)
                .unwrap();
            self.max7219
                .write_raw_byte(1, row as u8 + 1, byte2)
                .unwrap();
        } else {
            self.max7219
                .write_raw_byte(2, (row as u8 - 8) + 1, byte1)
                .unwrap();
            self.max7219
                .write_raw_byte(3, (row as u8 - 8) + 1, byte2)
                .unwrap();
        }
    }

    pub fn update_brightness(&mut self, brightness: u8) {
        for i in 0..NUM_DISPLAYS {
            self.max7219.set_intensity(i, brightness).unwrap();
        }
    }

    pub fn update_leds(&mut self, life: &Life) {
        let mut quad = [[0; HALF_ROWS]; NUM_DISPLAYS];

        for i in 0..HALF_ROWS {
            quad[0][i] = (life.cells[i] >> 8) as u8;
            quad[1][i] = (life.cells[i] & 0xFF) as u8;
            quad[2][i] = (life.cells[i + 8] >> 8) as u8;
            quad[3][i] = (life.cells[i + 8] & 0xFF) as u8;
        }

        for (i, bytes) in quad.iter().enumerate() {
            self.max7219.write_raw(i, bytes).unwrap();
        }
    }

    pub fn draw_xhair(&mut self, life: &Life, state: &State) {
        // The set_* functions use a skip value of MAX to know not to skip at all
        let mut skip_row = ROWS;
        let mut skip_col = COLS;

        // We want the center of the crosshair to reflect the state of the cell under it (on or off)
        if life.get_cell(state.xhair_col, state.xhair_row) == 0 {
            skip_row = state.xhair_row;
            skip_col = state.xhair_col;
        }

        // Make sure to set column first, otherwise it would erase the work done by set_row
        self.set_column(state.xhair_col, skip_row, life);
        self.set_row(state.xhair_row, skip_col);
    }

    pub fn new(spi: MatrixSpi, cs: MatrixCs) -> Self {
        let mut max7219 = MAX7219::from_spi_cs(NUM_DISPLAYS, spi, cs).unwrap();
        max7219.power_on().unwrap();
        Self { max7219 }
    }
}
