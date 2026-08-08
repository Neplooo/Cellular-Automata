mod rulesets {
    pub mod conway_ruleset;
    pub mod LwoD_ruleset;
    pub mod seeds_ruleset;
}


mod common;
mod renderer;

pub use crate::rulesets::conway_ruleset::simulate_conway_generation;
pub use crate::rulesets::LwoD_ruleset::simulate_lwod_generation;

pub use crate::common::initialize_grid;
pub use crate::common::display_grid;
pub use crate::common::check_alive_neighbors;

pub use crate::renderer::run_renderer;