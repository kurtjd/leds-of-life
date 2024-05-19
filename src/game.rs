const ROWS: usize = 16;
const COLS: usize = 16;

pub struct State {
    pub paused: bool,
    pub xhair_moved: bool,
    pub xhair_row: usize,
    pub xhair_col: usize,
}

impl State {
    pub fn new() -> Self {
        Self {
            paused: true,
            xhair_moved: true,
            xhair_row: ROWS / 2 - 1,
            xhair_col: COLS / 2 - 1,
        }
    }
}

pub struct Life {
    // Cells are represented as an array of 16-bit rows, where each bit is a cell (alive or dead)
    pub cells: [u16; ROWS],
}

impl Life {
    pub fn get_cell(&self, x: usize, y: usize) -> u8 {
        ((self.cells[y] >> (COLS - x - 1)) & 1) as u8
    }

    pub fn count_neighbors(&self, x: usize, y: usize) -> u8 {
        let mut total = 0;

        // A cell has 8 neighbors (with wraparound)
        for ny in y as i32 - 1..=y as i32 + 1 {
            for nx in x as i32 - 1..=x as i32 + 1 {
                let tmp_y = if ny < 0 {
                    ROWS - 1
                } else if ny as usize >= ROWS {
                    0
                } else {
                    ny as usize
                };

                let tmp_x = if nx < 0 {
                    COLS - 1
                } else if nx as usize >= COLS {
                    0
                } else {
                    nx as usize
                };

                // Don't count self as neighbor
                if tmp_y != y || tmp_x != x {
                    total += self.get_cell(tmp_x, tmp_y);
                }
            }
        }

        total
    }

    pub fn live(&mut self) {
        let mut new_cells = self.cells;

        for (y, cell_block) in new_cells.iter_mut().enumerate() {
            for x in 0..COLS {
                let neighbors = self.count_neighbors(x, y);

                /* If cell has fewer than 2 neighbors, die from loneliness.
                 * If cell has more than 3 neighbors, die from overcrowdedness.
                 * If cell is dead but has exactly 3 neighbors, come back to life!
                 */
                if !(2..=3).contains(&neighbors) {
                    *cell_block &= !(1 << (COLS - x - 1));
                } else if neighbors == 3 || (self.get_cell(x, y) == 1 && neighbors == 2) {
                    *cell_block |= 1 << (COLS - x - 1);
                }
            }
        }

        self.cells = new_cells;
    }

    // Hack the planet!
    pub fn draw_glider(&mut self) {
        self.cells[0] = 0x4000;
        self.cells[1] = 0x2000;
        self.cells[2] = 0xE000;
    }

    pub fn new() -> Self {
        Self { cells: [0; ROWS] }
    }
}
