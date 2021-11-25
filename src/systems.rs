use std::{
    borrow::BorrowMut,
    collections::{BinaryHeap, HashSet},
    hash::Hash,
};

use rand::Rng;
use raylib::prelude::*;
use shipyard::*;

use crate::{
    entities_components::{
        Position, RLHandle, RLThread, RecentlyTagged, TagState, Tagged, Time, Velocity,
    },
    HEIGHT, PLAYER_SIZE, WIDTH,
};

pub fn window_should_close(world: &World) -> bool {
    let ref rl = world.borrow::<UniqueViewMut<RLHandle>>().unwrap().0;
    rl.window_should_close()
}

/// When the world has ben initalized with players, we can simply tag the first agent
/// as "it", such that the games can begin!
pub fn tag_initial_random_player(mut tagged: ViewMut<Tagged>) {
    let mut tagged = tagged
        .borrow_mut()
        .iter()
        .next()
        .expect("No entity with tagged component");
    tagged.0 = TagState::It;
}

pub const FRAME: &str = "FRAME";

/// A collections of systems to run for each frame.
pub fn register_workloads(world: &World) {
    Workload::builder(FRAME)
        // Move players given their velocity every tick
        .with_system(update_player_position)
        // Play tag
        .with_system(tag_collided_players)
        // Clear recently tagged players
        .with_system(clear_old_recently_tagged)
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
        let mut vel = &mut vel.0;

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

fn tag_collided_players(
    positions: View<Position>,
    time: UniqueView<Time>,
    mut tagged: ViewMut<Tagged>,
    mut recently_tagged: UniqueViewMut<RecentlyTagged>,
) {
    // The execution time characteristics of the following is _not ideal_.
    // Collisions should ideally be handled through specialized data structures.
    // For example quad trees.

    // The currently, and only, tagged player
    let tagged_it = (&positions, &tagged)
        .iter()
        .with_id()
        .find(|(_, (_, tag))| tag.0 == TagState::It)
        .clone();

    if let Some((it_id, (it_pos, _))) = tagged_it {
        let mut have_tagged_new = false;

        for (id, (pos, mut tag)) in (&positions, &mut tagged).iter().with_id() {
            // The tagged player is touching another, non-tagged player
            if tag.0 != TagState::It
                && (&it_pos.0).distance_to(&pos.0) <= PLAYER_SIZE
                && !recently_tagged.0.contains_key(&id)
            {
                // Untag the player who is currently "it", tag the "non-it" player and stop.
                tag.0 = TagState::It;
                have_tagged_new = true;

                // Add both players from the interaction to the map of recently-tagged players
                recently_tagged.0.insert(id, time.0);
                recently_tagged.0.insert(it_id, time.0);
                break;
            }
        }

        // If the tagging has been replaced, update the now previously tagged player.
        if have_tagged_new {
            if let Ok(mut tagged) = (&mut tagged).get(it_id).as_mut() {
                tagged.0 = TagState::NotIt;
            };
        }
    }
}

// 5 seconds after tagging, players will be removed from the
// recently-tagged players map
const CLEAR_AFTER_SECONDS: u128 = 5 * 1000; // milliseconds
fn clear_old_recently_tagged(
    time: UniqueView<Time>,
    mut recently_tagged: UniqueViewMut<RecentlyTagged>,
) {
    // Only keep recent players if their timestamp is within the given constant
    recently_tagged
        .0
        .drain_filter(|_, timestamp| *timestamp + CLEAR_AFTER_SECONDS <= time.0);
}

/// Render player positions into the raylib framebuffer.
fn render_players(
    mut rlh: UniqueViewMut<RLHandle>,
    rlt: NonSendSync<UniqueView<RLThread>>,
    positions: View<Position>,
    velocities: View<Velocity>,
    tagged: View<Tagged>,
) {
    let mut d = rlh.0.begin_drawing(&rlt.0);

    d.clear_background(Color::WHITE);

    // Each player is shown with position as dot, line as direction they're facing
    for (pos, vel, tag) in (&positions, &velocities, &tagged).iter() {
        let pos = &pos.0;
        let vel = &vel.0;
        let tag = &tag.0;

        // The drawn direction vector is shown relative to the drawn player size.
        let direction_vector = vel.clone() * PLAYER_SIZE;

        // Players that are "it" will have a different color.
        let color = match tag {
            TagState::NotIt => Color::BLACK,
            TagState::It => Color::GOLD,
        };

        d.draw_circle(pos.x as i32, pos.y as i32, PLAYER_SIZE, color);
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
