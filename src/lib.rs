mod rulesets {
    pub mod LwoD_ruleset;
    pub mod brians_brain_ruleset;
    pub mod conway_ruleset;
    pub mod seeds_ruleset;
}

mod common;
mod renderer;

pub use crate::rulesets::LwoD_ruleset::simulate_lwod_generation;
pub use crate::rulesets::brians_brain_ruleset::init_brian_grid;
pub use crate::rulesets::brians_brain_ruleset::simulate_brian_generation;
pub use crate::rulesets::conway_ruleset::simulate_conway_generation;

pub use crate::common::check_alive_neighbors;
pub use crate::common::display_grid;
pub use crate::common::initialize_grid;

pub use crate::renderer::{Ruleset, run_renderer};
