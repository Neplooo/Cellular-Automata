use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use Cellular_Automata::run_renderer;

use crate::common::initialize_grid;
use crate::conway_ruleset::simulate_conway_generation;
use crate::LwoD_ruleset::simulate_lwod_generation;

mod common;
mod conway_ruleset;
mod LwoD_ruleset;

const GRID_SIZE: usize = 50;
const GENERATION_DELAY_MS: u64 = 500;

fn main() {
    let initial_grid = initialize_grid(GRID_SIZE, 0);
    let (sender, receiver) = mpsc::channel();
    let mut grid = initial_grid.clone();
    let mut gen_births: i32 = 0;
    let mut gen_deaths: i32 = 0;

    thread::spawn(move || loop {
        if sender.send(grid.clone()).is_err() {
            break;
        }
        (grid, gen_births, gen_deaths) = simulate_conway_generation(&grid, GRID_SIZE);

        if gen_births == 0 && gen_deaths == 0 {
            println!("No more births or deaths, simulation has stabilized.");
            break;
        }

        thread::sleep(Duration::from_millis(GENERATION_DELAY_MS));
    });

    run_renderer(initial_grid, receiver).expect("Renderer failed");
}
