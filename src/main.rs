use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use Cellular_Automata::{Ruleset, run_renderer};

mod common;
mod rulesets {
    pub mod LwoD_ruleset;
    pub mod brians_brain_ruleset;
    pub mod conway_ruleset;
    pub mod seeds_ruleset;
}

use crate::common::initialize_grid;
use crate::rulesets::LwoD_ruleset::simulate_lwod_generation;
use crate::rulesets::brians_brain_ruleset::{init_brian_grid, simulate_brian_generation};
use crate::rulesets::conway_ruleset::simulate_conway_generation;
use crate::rulesets::seeds_ruleset::simulate_seeds_generation;

const GRID_SIZE: usize = 200;
const GENERATION_DELAY_MS: u64 = 500;

fn main() {
    let initial_grid = init_brian_grid(GRID_SIZE, 0.7 ,false); //Max for conway grid init: 127
    let (sender, receiver) = mpsc::channel();
    let mut grid = initial_grid.clone();
    let mut gen_births: i32 = 0;
    let mut gen_deaths: i32 = 0;

    thread::spawn(move || {
        loop {
            if sender.send(grid.clone()).is_err() {
                break;
            }
            (grid, gen_births, gen_deaths) = simulate_brian_generation(&grid, GRID_SIZE);

            if gen_births == 0 && gen_deaths == 0 {
                println!("No more births or deaths, simulation has stabilized.");
                break;
            }

            thread::sleep(Duration::from_millis(GENERATION_DELAY_MS));
        }
    });

    run_renderer(initial_grid, receiver, Ruleset::BriansBrain).expect("Renderer failed");
}
