use raylib::prelude::*;
use shipyard::*;

use crate::entities_components::{Position, RLHandle, RLThread, Velocity};

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
        let norm_vec = vel.normalize() * 15.0;

        d.draw_circle(pos.0.x as i32, pos.0.y as i32, 5.0, Color::BLACK);
        d.draw_line(
            pos.0.x as i32,
            pos.0.y as i32,
            pos.0.x as i32 + norm_vec.0.x as i32,
            pos.0.y as i32 + norm_vec.0.y as i32,
            Color::BLUE,
        )
    }
}
