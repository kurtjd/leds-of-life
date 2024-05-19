const ROWS: u8 = 16;
const COLS: u8 = 16;

pub struct State {
    pub paused: bool,
    pub xhair_moved: bool,
    pub xhair_row: u8,
    pub xhair_col: u8,
}

impl State {
    pub fn new() -> Self {
        Self {
            paused: true,
            xhair_moved: true,
            xhair_row: 7,
            xhair_col: 7,
        }
    }
}

pub struct Life {
    pub cells: [u16; ROWS as usize],
}

impl Life {
    pub fn get_cell(&self, x: u8, y: u8) -> u8 {
        ((self.cells[y as usize] >> (COLS - x - 1)) & 1) as u8
    }

    pub fn count_neighbors(&self, x: u8, y: u8) -> u8 {
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

    pub fn live(&mut self) {
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

    pub fn draw_glider(&mut self) {
        self.cells[0] = 0x4000;
        self.cells[1] = 0x2000;
        self.cells[2] = 0xE000;
    }

    pub fn new() -> Self {
        Self {
            cells: [0; ROWS as usize],
        }
    }
}