#![feature(hash_drain_filter)]

use rand::prelude::*;
use shipyard::*;

pub mod behaviours;
pub mod entities_components;
pub mod systems;
pub use crate::entities_components::*;
pub use crate::systems::*;

const PLAYER_SIZE: f32 = 10.0;

/// Width of the `world` view
pub const WIDTH: i32 = 1024;
/// Height of the `world` view
pub const HEIGHT: i32 = 1024;

pub fn initialize_world(player_count: usize) -> World {
    let mut world = World::default();
    // Time is kept and updated after every frame/tick
    world.add_unique(Time::default()).unwrap();
    // Total number of tags shown in view
    world.add_unique(TagCount::default()).unwrap();
    // R*-Tree over all players used each frame
    world.add_unique(PlayersPositionRTree::default()).unwrap();

    let mut rng = rand::thread_rng();

    world
        .bulk_add_entity((0..player_count).map(|_| {
            (
                rng.gen::<Position>(),
                rng.gen::<Velocity>(),
                RecentlyTagged::default(),
                Tagged::default(),
                PlayerBehaviour::default(),
            )
        }))
        .next();

    world
        .run(tag_initial_random_player)
        .expect("one inital player to be tagged");

    register_workloads(&world);

    world
}
