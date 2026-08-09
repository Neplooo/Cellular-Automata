// Conway's Game of Life ruleset implementation
// https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life

use crate::common::check_alive_neighbors;
use array2d::{Array2D, Error};

pub fn simulate_conway_generation(
    grid: &Array2D<i32>,
    grid_size: usize,
) -> (Array2D<i32>, i32, i32) {
    let mut new_grid = Array2D::filled_with(0, grid_size, grid_size);

    let mut births = 0;
    let mut deaths = 0;

    for row in 0..grid_size {
        for col in 0..grid_size {
            //First, we need to count the number of neighbors that are alive (1)
            let alive_neighbors = check_alive_neighbors(grid, row, col);

            let curr_cell = grid.get(row, col).unwrap();

            //Now, based on the number of alive neighbors, we can apply Conway's rules to determine the state of the cell in the new grid.
            if curr_cell == &1 && (alive_neighbors < 2 || alive_neighbors > 3) {
                //If the cell is alive and is isolated/overpopulated
                new_grid.set(row, col, 0).ok(); //it dies.
                deaths += 1;
            } else if curr_cell == &0 && alive_neighbors == 3 {
                //If the cell is dead and has exactly 3 alive neighbors
                new_grid.set(row, col, 1).ok(); //it comes to life.
                births += 1;
            } else {
                new_grid.set(row, col, *curr_cell).ok(); //Otherwise, the cell remains in its current state (either alive or dead).
            }
        }
    }

    println!("Births: {}, Deaths: {}", births, deaths);

    (new_grid, births, deaths)
}
