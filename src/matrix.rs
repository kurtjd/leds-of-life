use stm32f1xx_hal::{
    gpio::{self, Alternate, Output},
    pac::SPI1,
    spi::{Mode, NoMiso, Phase, Polarity, Spi, Spi1NoRemap},
};

use max7219::{connectors::SpiConnectorSW, *};
use crate::game::*;

const ROWS: u8 = 16;
const COLS: u8 = 16;

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

impl Matrix {
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

    fn set_row(&mut self, row: u8, skip_col: u8) {
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

    pub fn update_brightness(&mut self, brightness: u8) {
        for i in 0..4 {
            self.max7219.set_intensity(i, brightness).unwrap();
        }
    }

    pub fn update_leds(&mut self, life: &Life) {
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

    pub fn draw_xhair(&mut self, life: &Life, state: &State) {
        let mut skip_row = ROWS;
        let mut skip_col = COLS;

        if life.get_cell(state.xhair_col, state.xhair_row) == 0 {
            skip_row = state.xhair_row;
            skip_col = state.xhair_col;
        }

        self.set_column(state.xhair_col, skip_row as usize, life);
        self.set_row(state.xhair_row, skip_col);
    }

    pub fn new(spi: MatrixSpi, cs: MatrixCs) -> Self {
        let mut max7219 = MAX7219::from_spi_cs(4, spi, cs).unwrap();
        max7219.power_on().unwrap();
        Self { max7219 }
    }
}
