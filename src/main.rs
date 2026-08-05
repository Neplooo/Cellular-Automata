use core::time;
use std::thread::sleep;

use array2d::{Array2D, Error};
use rand::{Rng, random};

mod common;
mod conway_ruleset;

use crate::common::{initialize_grid, display_grid};
use crate::conway_ruleset::simulate_conway_generation;

// Input Parameters
const GENERATIONS: i32 = 100;
const GRID_SIZE: usize = 20;




fn main() {
    let mut grid = initialize_grid(GRID_SIZE);


    println!("Starting Configuration:");

    display_grid(&grid, GRID_SIZE);

    for r#gen in 0..GENERATIONS {
        println!("Generation: {}", r#gen);
        grid = simulate_conway_generation(&grid, GRID_SIZE);
        display_grid(&grid, GRID_SIZE);
        sleep(time::Duration::from_millis(500));
        clearscreen::clear().expect("Failed to clear screen");
    }

}

