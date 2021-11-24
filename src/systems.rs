use raylib::prelude::*;
use shipyard::*;

use crate::{
    entities_components::{Position, RLHandle, RLThread, Velocity},
    PLAYER_SIZE,
};

pub fn window_should_close(world: &World) -> bool {
    let ref rl = world.borrow::<UniqueViewMut<RLHandle>>().unwrap().0;
    rl.window_should_close()
}

pub const FRAME: &str = "FRAME";

/// A collections of systems to run for each frame.
pub fn register_workloads(world: &World) {
    Workload::builder(FRAME)
        // Render the players at the end of the frame tick
        .with_system(render_players)
        .add_to_world(&world)
        .unwrap();
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
        let norm_vec = vel.normalize() * (PLAYER_SIZE * 2.0);

        d.draw_circle(pos.0.x as i32, pos.0.y as i32, PLAYER_SIZE, Color::BLACK);
        let ray_pos: Vector2 = pos.into();
        let ray_end_pos: Vector2 = Vector2 {
            x: ray_pos.x + norm_vec.0.x,
            y: ray_pos.y + norm_vec.0.y,
        };
        d.draw_line_ex(
            ray_pos,
            ray_end_pos,
            f32::max(PLAYER_SIZE * 0.5, 1.0),
            Color::BLUE,
        )
    }
}
