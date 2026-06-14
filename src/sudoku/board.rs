const BOARD_N: usize = 3;

pub type CellType = u16;

#[derive(Debug, Clone, Copy)]
pub struct SudokuCell {
    pub value: Option<CellType>,
    pub editable: bool,
    pub x: usize,
    pub y: usize,
}

impl SudokuCell {
    fn new(value: Option<CellType>) -> Self {
        SudokuCell {
            value,
            editable: value.is_none(),
            x: 0,
            y: 0,
        }
    }
}

#[derive(Debug)]
pub struct SudokuBoard {
    cells: [[SudokuCell; BOARD_MAX_NUMBER]; BOARD_MAX_NUMBER],
}

const BOARD_MAX_NUMBER: usize = BOARD_N * BOARD_N;

impl SudokuBoard {
    pub const BOARD_N: usize = BOARD_N;
    pub const BOARD_MAX_NUMBER: usize = BOARD_MAX_NUMBER;

    pub fn valid_values() -> Vec<CellType> {
        (1..=Self::BOARD_MAX_NUMBER as CellType).collect()
    }

    fn initialize_board() -> [[SudokuCell; BOARD_MAX_NUMBER]; BOARD_MAX_NUMBER] {
        [[SudokuCell::new(None); BOARD_MAX_NUMBER]; BOARD_MAX_NUMBER]
    }

    pub fn new(list: Vec<Vec<Option<CellType>>>) -> Result<Self, String> {
        if list.len() != Self::BOARD_MAX_NUMBER {
            return Err("The provided list must have 9 lines".to_string());
        }

        let mut sudoku_board = SudokuBoard {
            cells: Self::initialize_board(),
        };

        for (line_index, row) in list.iter().enumerate() {
            if row.len() != Self::BOARD_MAX_NUMBER {
                return Err("The provided list must have 9 lines".to_string());
            }

            for (column_index, value) in row.iter().enumerate() {
                let cell = &mut sudoku_board.cells[line_index][column_index];
                cell.value = *value;
                cell.editable = value.is_none();
                cell.x = line_index;
                cell.y = column_index;
            }
        }

        Ok(sudoku_board)
    }

    pub fn find_cell_from_coordinates(&self, x: usize, y: usize) -> Result<&SudokuCell, String> {
        if x >= Self::BOARD_MAX_NUMBER || y >= Self::BOARD_MAX_NUMBER {
            return Err(format!("Invalid coordinates ({}, {})", x, y));
        }

        Ok(&self.cells[x][y])
    }

    pub fn set_cell(
        &mut self,
        x: usize,
        y: usize,
        value: Option<CellType>,
    ) -> Result<SudokuCell, String> {
        if x >= Self::BOARD_MAX_NUMBER || y >= Self::BOARD_MAX_NUMBER {
            return Err(format!("Invalid coordinates ({}, {})", x, y));
        }

        if !self.is_valid_insertion(x, y, value) {
            return Err("Invalid Insertion".to_string());
        }

        self.cells[x][y].value = value;
        Ok(self.cells[x][y])
    }

    pub fn set_cell_unchecked(
        &mut self,
        x: usize,
        y: usize,
        value: Option<CellType>,
    ) -> Result<(), String> {
        if x >= Self::BOARD_MAX_NUMBER || y >= Self::BOARD_MAX_NUMBER {
            return Err(format!("Invalid coordinates ({}, {})", x, y));
        }

        self.cells[x][y].value = value;

        Ok(())
    }

    /// Sum of digits 1–BOARD_MAX_NUMBER missing from a filled line (row or column).
    fn line_penalty(values: &[Option<CellType>]) -> Result<u16, String> {
        let mut present = [false; Self::BOARD_MAX_NUMBER + 1];

        for value in values {
            let Some(v) = value else {
                return Err("Line contains an empty cell".to_string());
            };
            let index = *v as usize;
            if index == 0 || index >= present.len() {
                return Err(format!("Invalid cell value {v}"));
            }
            present[index] = true;
        }

        Ok((1..=Self::BOARD_MAX_NUMBER as CellType)
            .filter(|&d| !present[d as usize])
            .sum())
    }

    pub(crate) fn row_cost(&self, row: usize) -> Result<u16, String> {
        if row >= Self::BOARD_MAX_NUMBER {
            return Err(format!("Invalid row {row}"));
        }
        let values: [Option<CellType>; Self::BOARD_MAX_NUMBER] =
            std::array::from_fn(|col| self.cells[row][col].value);
        Self::line_penalty(&values)
    }

    pub(crate) fn column_cost(&self, col: usize) -> Result<u16, String> {
        if col >= Self::BOARD_MAX_NUMBER {
            return Err(format!("Invalid column {col}"));
        }
        let values: [Option<CellType>; Self::BOARD_MAX_NUMBER] =
            std::array::from_fn(|row| self.cells[row][col].value);
        Self::line_penalty(&values)
    }

    pub fn calculate_raw_column_cost(&self) -> Result<u16, String> {
        let mut cost = 0;

        for x in 0..Self::BOARD_MAX_NUMBER {
            cost += self.row_cost(x)?;
            cost += self.column_cost(x)?;
        }

        Ok(cost)
    }

    fn calculate_final_cost(&self) -> Result<u16, String> {
        let row_and_column_cost = self.calculate_raw_column_cost()?;

        let box_cost = {
            let mut cost = 0;

            for box_row in 0..Self::BOARD_N {
                for box_col in 0..Self::BOARD_N {
                    let mut missing_values_in_box = Self::valid_values();
                    let start_x = box_row * Self::BOARD_N;
                    let start_y = box_col * Self::BOARD_N;

                    for x in start_x..start_x + Self::BOARD_N {
                        for y in start_y..start_y + Self::BOARD_N {
                            let value = self.cells[x][y].value;

                            if value.is_none() {
                                return Err(format!("Cell {}, {} is empty", x, y));
                            }

                            missing_values_in_box.retain(|&v| value.unwrap() != v);
                        }
                    }

                    cost += missing_values_in_box.iter().sum::<CellType>();
                }
            }

            cost
        };

        let total = row_and_column_cost
            .checked_add(box_cost)
            .ok_or_else(|| "Cost overflow".to_string())?;

        Ok(total)
    }

    pub fn validate_solution(&self) -> Result<(), String> {
        let cost = self.calculate_final_cost()?;

        if cost > 0 {
            return Err("Sudoku does not have a optimal solution".to_string());
        }

        Ok(())
    }

    pub fn is_valid_insertion(&self, x: usize, y: usize, new_value: Option<CellType>) -> bool {
        if let Some(value) = new_value {
            self.is_valid_box(x, y, value)
                && self.is_valid_line(x, value)
                && self.is_valid_column(y, value)
        } else {
            true
        }
    }

    fn is_valid_box(&self, x: usize, y: usize, new_value: CellType) -> bool {
        let start_x = (x / Self::BOARD_N) * Self::BOARD_N;
        let start_y = (y / Self::BOARD_N) * Self::BOARD_N;

        for i in start_x..start_x + Self::BOARD_N {
            for j in start_y..start_y + Self::BOARD_N {
                if self.cells[i][j].value == Some(new_value) {
                    return false;
                }
            }
        }

        true
    }

    fn is_valid_line(&self, x: usize, new_value: CellType) -> bool {
        for y in 0..Self::BOARD_MAX_NUMBER {
            if self.cells[x][y].value == Some(new_value) {
                return false;
            }
        }

        true
    }

    fn is_valid_column(&self, y: usize, new_value: CellType) -> bool {
        for x in 0..Self::BOARD_MAX_NUMBER {
            if self.cells[x][y].value == Some(new_value) {
                return false;
            }
        }

        true
    }

    pub fn get_editable_cells(&self) -> Vec<(usize, usize)> {
        let mut editable_cells = vec![];

        for x in 0..Self::BOARD_MAX_NUMBER {
            for y in 0..Self::BOARD_MAX_NUMBER {
                let cell = &self.cells[x][y];

                if cell.editable {
                    editable_cells.push((cell.x, cell.y));
                }
            }
        }

        editable_cells
    }

    pub fn get_cells_from_box(&self, box_i: usize, box_j: usize) -> Vec<&SudokuCell> {
        let mut cells = Vec::with_capacity(Self::BOARD_N * Self::BOARD_N);

        for x in 0..Self::BOARD_N {
            for y in 0..Self::BOARD_N {
                cells.push(&self.cells[box_i * Self::BOARD_N + x][box_j * Self::BOARD_N + y]);
            }
        }

        cells
    }

    pub fn all_cells(&self) -> Vec<&SudokuCell> {
        let mut cells = Vec::with_capacity(Self::BOARD_MAX_NUMBER * Self::BOARD_MAX_NUMBER);

        for x in 0..Self::BOARD_MAX_NUMBER {
            for y in 0..Self::BOARD_MAX_NUMBER {
                cells.push(&self.cells[x][y]);
            }
        }

        cells
    }
}
