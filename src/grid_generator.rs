use rand::Rng;
use crate::grid::{GameGrid, screen_to_grid};
use crate::GPUSprite;



pub fn generate_grid(mut x: f32, mut y: f32, game_grid: &mut GameGrid, sprites: &mut Vec<GPUSprite>) {
    let mut color_counters = [0; 6];
    // loop through every slot in the grid
    for _col in 0..10 {
        for _row in 0..20 {
            // generate a random number between 0 and 2. This number will be used to select the color of the sprite
            let mut rng = rand::thread_rng();
            let mut random_number: u32 = rng.gen_range(0..=5);
            let mut sprite: GPUSprite = GPUSprite {
                screen_region: [0.0, 0.0, 8.0, 8.0],
                sheet_region: [0.0 / 80.0, 0.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
            };

            // convert pixels to grid units
            let (grid_x, grid_y) = screen_to_grid(x, y);

            // println!("{}", random_number);

            // Check if placing more than three of the same color in a row
            if color_counters[random_number as usize] >= 3 {
                // If more than three, select a different color
                let mut new_random_number = random_number;
                while new_random_number == random_number {
                    new_random_number = rng.gen_range(0..=5);
                }
                random_number = new_random_number;
            }

            // Increment the counter for the chosen color
            color_counters[random_number as usize] += 1;

            // Reset the counters for other colors
            for i in 0..6 {
                if i != random_number as usize {
                    color_counters[i] = 0;
                }
            }

            // set the color of the sprite according to the random number generated
            match random_number {
                0 => {
                    // set the color of the sprite to white
                    sprite = GPUSprite {
                        screen_region: [x, y, 8.0, 8.0],
                        sheet_region: [0.0 / 80.0, 0.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
                    };
                    game_grid.fill_space(grid_x, grid_y, "white");
                }
                1 => {
                    // set the color of the sprite to dark blue
                    sprite = GPUSprite {
                        screen_region: [x, y, 8.0, 8.0],
                        sheet_region: [0.0 / 80.0, 16.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
                    };
                    game_grid.fill_space(grid_x, grid_y, "dark blue");
                }
                2 => {
                    // set the color of the sprite to light blue
                    sprite = GPUSprite {
                        screen_region: [x, y, 8.0, 8.0],
                        sheet_region: [0.0 / 80.0, 32.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
                    };
                    game_grid.fill_space(grid_x, grid_y, "light blue");
                }
                3 => {
                    // set the color of the sprite to light orange
                    sprite = GPUSprite {
                        screen_region: [x, y, 8.0, 8.0],
                        sheet_region: [0.0 / 80.0, 48.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
                    };
                    game_grid.fill_space(grid_x, grid_y, "light orange");
                }
                4 => {
                    // set the color of the sprite to dark orange
                    sprite = GPUSprite {
                        screen_region: [x, y, 8.0, 8.0],
                        sheet_region: [0.0 / 80.0, 64.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
                    };
                    game_grid.fill_space(grid_x, grid_y, "dark orange");
                }
                5 => {
                    // set the color of the sprite to white with orange
                    sprite = GPUSprite {
                        screen_region: [x, y, 8.0, 8.0],
                        sheet_region: [0.0 / 80.0, 80.0 / 160.0, 8.0 / 80.0, 8.0 / 160.0],
                    };
                    game_grid.fill_space(grid_x, grid_y, "white orange");
                }
                _ => println!("Random number is out of range"),
            }

            // game_grid.print_space(grid_x, grid_y);
            sprites.push(sprite);
            y -= 8.0;
        }

        x += 8.0;
        y = 152.0;
    }
}