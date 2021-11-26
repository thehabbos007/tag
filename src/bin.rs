use std::env;
use std::rc::Rc;
use std::time::SystemTime;

use shipyard::*;

use tag::entities_components::*;
use tag::systems::*;
use tag::{initialize_world, HEIGHT, WIDTH};

fn main() {
    let args: Vec<String> = env::args().collect();

    // Number of players in `world`
    let player_count = if let Some(Ok(player_count)) = args.get(1).map(|x| str::parse::<usize>(&x))
    {
        player_count
    } else {
        panic!("First argument given is not a number. Please enter number of players.");
    };

    let (mut rl, thread) = raylib::init()
        .size(WIDTH as i32, HEIGHT as i32)
        .title("Tag!")
        .build();
    rl.set_target_fps(60);

    let world = initialize_world(player_count);

    world.add_unique(RLHandle(rl)).unwrap();
    world.add_unique_non_send_sync(RLThread(thread)).unwrap();

    let world = Rc::new(world);

    loop {
        // Main loop is checking the window close state
        if window_should_close(&world) {
            break;
        }

        // And advancing the ECS one "tick" and "frame" at a time.
        world.run_workload(TICK).unwrap();
        world.run_workload(FRAME).unwrap();

        // Change timestamp after every frame
        world.borrow::<UniqueViewMut<Time>>().unwrap().0 = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("time to be readable")
            .as_millis();
    }
}
