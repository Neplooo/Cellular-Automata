mod conway_ruleset;
mod LwoD_ruleset;
mod common;
mod renderer;

pub use crate::conway_ruleset::simulate_conway_generation;
pub use crate::LwoD_ruleset::simulate_lwod_generation;

pub use crate::common::initialize_grid;
pub use crate::common::display_grid;
pub use crate::common::check_alive_neighbors;

pub use crate::renderer::run_renderer;