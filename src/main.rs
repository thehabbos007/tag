use std::rc::Rc;

use rand::prelude::*;
use shipyard::*;

mod entities_components;
mod systems;
use crate::entities_components::*;
use crate::systems::*;

/// Number of players in `world`
const PLAYER_COUNT: usize = 50;
const PLAYER_SIZE: f32 = 10.0;
/// Width of the `world` view
const WIDTH: u16 = 1024;
/// Height of the `world` view
const HEIGHT: u16 = 1024;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Hello, World")
        .build();
    rl.set_target_fps(60);

    let mut world = World::default();
    world.add_unique(RLHandle(rl)).unwrap();
    world.add_unique_non_send_sync(RLThread(thread)).unwrap();
    let mut rng = rand::thread_rng();

    world
        .bulk_add_entity(
            (0..PLAYER_COUNT)
                .map(|_| (Position(rng.gen::<Point2D>()), Velocity(rng.gen::<Vec2d>()))),
        )
        .next();

    let world = Rc::new(world);
    register_workloads(&world);

    loop {
        // Main loop is checking the window close state
        if window_should_close(&world) {
            break;
        }
        // And advancing the ECS one "tick" or "frame" at a time.
        world.run_workload(FRAME).unwrap();
    }
}
