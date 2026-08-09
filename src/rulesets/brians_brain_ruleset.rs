// Brian's Brain ruleset implementation
// https://conwaylife.com/wiki/OCA:Brian%27s_Brain

use crate::common::check_alive_neighbors;
use array2d::Array2D;
use rand::{self, RngExt};

pub fn init_brian_grid(grid_size: usize, zoom_factor:f64, cascade: bool) -> Array2D<i32> {
    // Four States should exist. 0 = empty, 1 = alive, 2 = dying, 3 = dead

    let mut grid = Array2D::filled_with(0, grid_size, grid_size);

    let scaled_grid_size = ((grid_size as f64) * zoom_factor) as usize;
    let starting_iterator_scaled = grid_size - scaled_grid_size;

    match cascade {
        false => {
            let mut rng = rand::rng();

            for row in starting_iterator_scaled..scaled_grid_size {
                for col in starting_iterator_scaled..scaled_grid_size {
                    let random_state = rng.random_range(0..=1); //Each cell will either be empty or alive.

                    grid.set(row, col, random_state).ok();
                }
            }
        }
        true => {
            grid.set(grid_size / 2, grid_size / 2, 1).ok();
            grid.set(grid_size / 2, grid_size / 2 + 1, 1).ok();
            grid.set(grid_size / 2 + 1, grid_size / 2, 1).ok();
            grid.set(grid_size / 2 + 1, grid_size / 2 + 1, 1).ok();
        }
    };

    grid
}

pub fn simulate_brian_generation(
    grid: &Array2D<i32>,
    grid_size: usize,
) -> (Array2D<i32>, i32, i32) {
    let mut new_grid = Array2D::filled_with(0, grid_size, grid_size);
    let mut alive_count = 0;
    let mut dead_count = 0;

    for row in 0..grid_size {
        for col in 0..grid_size {
            let current_state = grid.get(row, col).unwrap();
            let alive_neighbors = check_alive_neighbors(grid, row, col);
            let new_cell = match current_state {
                0 => {
                    //If the cell has exactly 2 alive neighbors, it becomes alive. Otherwise, it remains empty.
                    if alive_neighbors == 2 {
                        alive_count += 1;
                        1
                    } else {
                        0
                    }
                }
                1 => {
                    2 //If the cell is alive, it moves to the dying state in the next generation.
                }
                2 => {
                    dead_count += 1;
                    3 //If the cell is dying, it moves to the dead-state and is "marked" for visualization purposes. It will be treated as dead in the next generation.
                }
                3 => {
                    //Same conditions as the empty cell, but the cell stays marked if it can't be revived.
                    if alive_neighbors == 2 {
                        alive_count += 1;
                        1
                    } else {
                        3
                    }
                }
                _ => 0, //Default case, should never be reached.
            };

            new_grid.set(row, col, new_cell).ok();
        }
    }

    println!("Alive Cells: {}, Dead Cells: {}", alive_count, dead_count);

    (new_grid, alive_count, dead_count)
}
