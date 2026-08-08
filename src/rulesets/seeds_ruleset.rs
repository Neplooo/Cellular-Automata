// Seeds ruleset implementation
// https://conwaylife.com/wiki/OCA:Seeds

use array2d::{Array2D, Error};
use crate::common::{check_alive_neighbors};

pub fn simulate_seeds_generation(grid: &Array2D<i32>, grid_size: usize) -> (Array2D<i32>, i32, i32) {

    let mut new_grid = Array2D::filled_with(0, grid_size, grid_size);

    let mut births = 0;
    let mut deaths = 0;

    for row in 0..grid_size {
        for col in 0..grid_size {

            //Check how many alive neighbors the current cell has.
            let alive_neighbors = check_alive_neighbors(grid, row, col);

            //implementing the rules of Life Without Death:
            let curr_cell = grid.get(row, col).unwrap();

            if curr_cell == &0 && alive_neighbors == 2 { //If the cell is dead and has exactly 2 alive neighbors, it comes to life.
                new_grid.set(row, col, 1).ok();
                births += 1;
            } else { //In any other scenario, the cell dies (either alive or dead). In Seeds, once a cell is alive, it dies in the next generation.
                new_grid.set(row, col, 0).ok();
                if curr_cell == &1 {
                    deaths += 1;
                }
            }

        }
    }

    println!("Births: {}, Deaths: {}", births, deaths);

    (new_grid, births, deaths)
}