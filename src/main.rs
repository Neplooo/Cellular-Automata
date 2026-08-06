use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use Cellular_Automata::run_renderer;

use crate::common::initialize_grid;
use crate::conway_ruleset::simulate_conway_generation;

mod common;
mod conway_ruleset;

const GRID_SIZE: usize = 20;
const GENERATION_DELAY_MS: u64 = 2000;

fn main() {
    let initial_grid = initialize_grid(GRID_SIZE);
    let (sender, receiver) = mpsc::channel();
    let mut grid = initial_grid.clone();

    thread::spawn(move || loop {
        if sender.send(grid.clone()).is_err() {
            break;
        }
        grid = simulate_conway_generation(&grid, GRID_SIZE);
        thread::sleep(Duration::from_millis(GENERATION_DELAY_MS));
    });

    run_renderer(initial_grid, receiver).expect("Renderer failed");
}
