mod it;
mod not_it;

use crate::entities_components::RTreeData;

use super::{Position, Velocity};
pub use it::*;
pub use not_it::*;

/// In order to evaluate the behaviour of a player, some information about
/// the player's environment is passed down to the bahviour specifications.
pub struct BehaviourContext<'a> {
    pub current_player: (&'a Position, &'a mut Velocity),
    pub distance_to_it: f32,
    pub nearest_5_neighbors: Vec<&'a RTreeData>,
}

/// A behaviour defines how an actor orients itself in accordance to the current
/// environment
pub trait BehaviourAction {
    fn revise_orientation(&self, ctx: BehaviourContext);
}
