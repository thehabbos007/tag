use rand::Rng;
use raylib::prelude::*;
use shipyard::*;

use crate::{
    entities_components::{Position, RLHandle, RLThread, Velocity},
    HEIGHT, PLAYER_SIZE, WIDTH,
};

pub fn window_should_close(world: &World) -> bool {
    let ref rl = world.borrow::<UniqueViewMut<RLHandle>>().unwrap().0;
    rl.window_should_close()
}

pub const FRAME: &str = "FRAME";

/// A collections of systems to run for each frame.
pub fn register_workloads(world: &World) {
    Workload::builder(FRAME)
        // Move players given their velocity every tick
        .with_system(update_player_position)
        // Render the players at the end of the frame tick
        .with_system(render_players)
        .add_to_world(&world)
        .unwrap();
}

/// The players are currently "simple" they wander aimlessly without any logic
/// Each player would be more sophisticated in later versions, such as different tactics
/// and different attributes.
fn update_player_position(mut positions: ViewMut<Position>, mut velocities: ViewMut<Velocity>) {
    let mut rng = rand::thread_rng();

    for (pos, vel) in (&mut positions, &mut velocities).iter() {
        let mut pos = &mut pos.0;
        let vel = &mut vel.0;

        // A player will wrap around if they hit corners 2d-game style
        pos.x = (pos.x + (vel.x)).rem_euclid(WIDTH);
        pos.y = (pos.y + (vel.y)).rem_euclid(HEIGHT);

        // Every now and then the player's direction changes
        if rng.gen_bool(0.005) {
            vel.x = (vel.x + rng.gen_range(-1..1)) % 5;
            vel.y = (vel.y + rng.gen_range(-1..1)) % 5;
        }
    }
}

/// Render player positions into the raylib framebuffer.
fn render_players(
    mut rlh: UniqueViewMut<RLHandle>,
    rlt: NonSendSync<UniqueView<RLThread>>,
    positions: View<Position>,
    velocities: View<Velocity>,
) {
    let mut d = rlh.0.begin_drawing(&rlt.0);

    d.clear_background(Color::WHITE);

    // Each player is shown with position as dot, line as direction they're facing
    for (pos, vel) in (&positions, &velocities).iter() {
        let pos = &pos.0;
        let vel = &vel.0;

        // The drawn direction vector is shown relative to the drawn player size.
        let direction_vector = vel.clone() * PLAYER_SIZE;

        d.draw_circle(pos.x as i32, pos.y as i32, PLAYER_SIZE, Color::BLACK);
        let ray_pos: Vector2 = pos.into();
        let ray_end_pos: Vector2 = Vector2 {
            x: ray_pos.x + (direction_vector.x as f32),
            y: ray_pos.y + (direction_vector.y as f32),
        };
        d.draw_line_ex(
            ray_pos,
            ray_end_pos,
            f32::max(PLAYER_SIZE * 0.5, 1.0),
            Color::BLUE,
        )
    }
}
