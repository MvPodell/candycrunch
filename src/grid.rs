#[derive(Copy, Clone)]
struct Space {
    color: &'static str, // Use a string slice for color
    filled: bool,
}

impl Space {
    fn new(color: &'static str) -> Self {
        Space {
            color,
            filled: false,
        }
    }
}

pub struct GameGrid {
    grid: [[Space; 10]; 20],
}

pub fn screen_to_grid(x: f32, y: f32) -> (usize, usize) {
    let grid_x = (x as usize - 80) / 8;
    let grid_y = 19 - (y as usize) / 8;
    (grid_x, grid_y)
}

impl GameGrid {
    pub fn new() -> Self {
        let mut grid = [[Space::new("empty"); 10]; 20];

        // Initialize the grid with space objects
        let mut y_cord = 518.0;
        let mut x_cord = 98.0;
        for row in 0..20 {
            for col in 0..10 {
                grid[row][col] = Space::new("nah");
                y_cord -= 88.0;
            }
            x_cord += 98.0;
            y_cord = 518.0;
        }

        GameGrid { grid }
    }

    pub fn print_grid(&self) {
        for row in &self.grid {
            for space in row {
                if space.filled {
                    print!("1 "); // You can change this to any character or representation for filled spaces
                } else {
                    print!("0 "); // You can change this to any character or representation for empty spaces
                }
            }
            println!();
        }
    }

    pub fn fill_space(&mut self, x: usize, y: usize, color: &'static str) {
        if x < 10 && y < 20 {
            self.grid[y][x].filled = true;
            self.grid[y][x].color = color;
        }
    }

    pub fn print_space(&self, x: usize, y: usize) {
        if x < 10 && y < 20 {
            let space = &self.grid[y][x];
            println!(
                "x: {}, y: {}, color: {}, filled: {}",
                x, y, space.color, space.filled
            );
        } else {
            println!("Invalid indices");
        }
    }

    // pub fn point_color(&self, grid_x: usize, grid_y: usize) -> &'static str {
    //     self.grid[grid_y][grid_x].color
    // }

    pub fn swap_colors(&mut self, x_coord: f32, y_coord: f32, last_clicked: (f32, f32)) {
        let (last_x, last_y) = last_clicked;

        let color1 = self.grid[y_coord as usize][x_coord as usize].color;
        let color2 = self.grid[last_y as usize][last_x as usize].color;
        // swap the colors
        self.grid[y_coord as usize][x_coord as usize].color = color2;
        self.grid[last_y as usize][last_x as usize].color = color1;
    }

    pub fn get_color_coords(&self, sprite_col: usize, sprite_row: usize) -> [f32; 4] {
        let color = match self.grid[sprite_col][sprite_row].color {
            "white" => [0.0, 0.0, 8.0 / 80.0, 8.0 / 160.0],
            "dark blue" => [0.0, 16.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
            "light blue" => [0.0 / 23.0, 32.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
            "light orange" => [0.0 / 80.0, 48.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
            "dark orange" => [0.0 / 80.0, 64.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
            "white orange" => [0.0 / 80.0, 80.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
            "black" => [0.0 / 80.0, 96.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
            // Add cases for other colors
            _ => [0.0, 0.0, 8.0 / 80.0, 8.0 / 160.0], // Default to a color
        };
        color
    }

    pub fn color_is_black(&self, x: usize, y: usize) -> bool {
        if x < 10 && y < 20 {
            let color = self.grid[y][x].color;
            return color == "black";
        } else {
            false
        }
    }

    pub fn set_black(&mut self, x: usize, y: usize) {
        let color = "black";
        self.grid[x][y].color = color;
        // println!("color after {} for ({}, {})", self.grid[x][y].color, x, y);
    }

    pub fn check_blackout_horiz(&self) -> (usize, usize, usize) {
        for row in 0..20 {
            let mut consecutive_count = 1;
            let mut last_color = "black";

            for index in 0..10 {
                let space = &self.grid[row][index];
                // println!("{last_color}");
                if space.filled && space.color == last_color && last_color != "black" {
                    consecutive_count += 1;
                    if consecutive_count == 4 {
                        // println!("row{row} index{index}");
                        if index < 4 {
                            return (row + 1, row, index);
                        } else {
                            return (((index - 3) * 20) + row + 1, row, index - 3);
                        }
                    }
                } else {
                    consecutive_count = 1;
                    last_color = space.color;
                }
            }
        }
        // return 202 to signify that there are not 4 in a row
        return (202, 0, 0);
    }
    pub fn check_blackout_vert(&self) -> (usize, usize, usize) {
        // Check vertically
        for col in 0..10 {
            let mut consecutive_count = 1;
            let mut last_color = "black";

            for row in 0..20 {
                let space = &self.grid[row][col];

                if space.filled && space.color == last_color && last_color != "black" {
                    consecutive_count += 1;
                    if consecutive_count == 4 {
                        // return the first index of the four in a column
                        return ((row - 2) + (20 * col), row - 3, col);
                    }
                } else {
                    consecutive_count = 1;
                    last_color = space.color;
                }
            }
        }
        // return 202 to signify that there are not 4 in a column
        return (202, 0, 0); // No four consecutive spaces found
    }
}