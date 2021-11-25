#![feature(hash_drain_filter)]

use std::collections::HashMap;
use std::collections::HashSet;
use std::rc::Rc;
use std::time::SystemTime;

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
const WIDTH: i16 = 1024;
/// Height of the `world` view
const HEIGHT: i16 = 1024;

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Tag!")
        .build();
    rl.set_target_fps(60);

    let mut world = World::default();
    world.add_unique(RLHandle(rl)).unwrap();
    world.add_unique_non_send_sync(RLThread(thread)).unwrap();

    // Time is kept and updated after every frame/tick
    world.add_unique(Time(0)).unwrap();
    // RecentlyTagged players map Entity Ids to a timestamp
    world.add_unique(RecentlyTagged(HashMap::new())).unwrap();

    let mut rng = rand::thread_rng();

    world
        .bulk_add_entity((0..PLAYER_COUNT).map(|_| {
            (
                rng.gen::<Position>(),
                rng.gen::<Velocity>(),
                Tagged(TagState::default()),
            )
        }))
        .next();

    world
        .run(tag_initial_random_player)
        .expect("one inital player to be tagged");

    let world = Rc::new(world);
    register_workloads(&world);

    loop {
        // Main loop is checking the window close state
        if window_should_close(&world) {
            break;
        }

        // And advancing the ECS one "tick" or "frame" at a time.
        world.run_workload(FRAME).unwrap();

        // Change timestamp after every frame
        world.borrow::<UniqueViewMut<Time>>().unwrap().0 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("time to be readable")
            .as_millis();
    }
}
