// Life Without Death ruleset implementation
// https://en.wikipedia.org/wiki/Life_without_Death

use crate::common::check_alive_neighbors;
use array2d::{Array2D, Error};

pub fn simulate_lwod_generation(grid: &Array2D<i32>, grid_size: usize) -> (Array2D<i32>, i32, i32) {
    let mut new_grid = Array2D::filled_with(0, grid_size, grid_size);

    let mut births = 0;
    let mut deaths = 0;

    for row in 0..grid_size {
        for col in 0..grid_size {
            //Check how many alive neighbors the current cell has.
            let alive_neighbors = check_alive_neighbors(grid, row, col);

            //implementing the rules of Life Without Death:
            let curr_cell = grid.get(row, col).unwrap();

            if curr_cell == &0 && alive_neighbors == 3 {
                //If the cell is dead and has exactly 3 alive neighbors, it comes to life.
                new_grid.set(row, col, 1).ok();
                births += 1;
            } else {
                //In any other scenario, the cell remains in its current state (either alive or dead). In Life Without Death, once a cell is alive, it never dies.
                new_grid.set(row, col, *curr_cell).ok();
            }
        }
    }

    (new_grid, births, deaths)
}
