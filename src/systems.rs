use std::borrow::BorrowMut;

use raylib::prelude::*;
use shipyard::*;
use spade::rtree::RTree;

use crate::{
    behaviours::{BehaviourAction, BehaviourContext},
    entities_components::{
        PlayerBehaviour, PlayersPositionRTree, Position, RLHandle, RLThread, RTreeData,
        RecentlyTagged, TagState, Tagged, Time, Velocity,
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
        .with_system(update_player_position_rtee)
        .with_system(commit_player_behaviour)
        // Play tag
        .with_system(tag_collided_players)
        // Clear recently tagged players
        .with_system(clear_old_recently_tagged)
        // Render the players at the end of the frame tick
        .with_system(render_players)
        .add_to_world(&world)
        .unwrap();
}

/// Move players in accordance to their velocity
fn update_player_position(v_velocity: View<Velocity>, mut vm_position: ViewMut<Position>) {
    for (pos, vel) in (&mut vm_position, &v_velocity).iter() {
        let geo_pos = &mut pos.0;
        let geo_vel = &vel.0;

        // A player will wrap around if they hit corners 2d-game style
        geo_pos[0] = (geo_pos[0] + (geo_vel[0])).rem_euclid(WIDTH as f32);
        geo_pos[1] = (geo_pos[1] + (geo_vel[1])).rem_euclid(HEIGHT as f32);
    }
}

/// Generate R*-Tree of all points - making nearest neighbour searches a breeze
fn update_player_position_rtee(
    v_position: ViewMut<Position>,
    v_velocity: View<Velocity>,
    v_tagged: View<Tagged>,
    v_recently_tagged: View<RecentlyTagged>,
    mut uvm_player_pos_rtree: UniqueViewMut<PlayersPositionRTree>,
) {
    uvm_player_pos_rtree.0 = RTree::bulk_load(
        (&v_position, &v_velocity, &v_recently_tagged, &v_tagged)
            .iter()
            .map(|(pos, vel, recently_tagged, tag)| RTreeData {
                position: pos.clone(),
                velocity: vel.clone(),
                recently_tagged: recently_tagged.0.is_some(),
                tagged: tag.clone(),
            })
            .collect(),
    );
}

/// Generate R*-Tree of all points - making nearest neighbour searches a breeze
fn commit_player_behaviour(
    v_position: View<Position>,
    v_tagged: View<Tagged>,
    v_player_behaviour: View<PlayerBehaviour>,
    uv_player_rtree: UniqueView<PlayersPositionRTree>,
    mut vm_velocity: ViewMut<Velocity>,
) {
    for (pos, vel, tag, behaviour) in (
        &v_position,
        &mut vm_velocity,
        &v_tagged,
        &v_player_behaviour,
    )
        .iter()
    {
        let tag = &tag.0;

        // 6 nearest neighbors including self
        // But exclude first, as this will be the current point.
        let nearest_5_neighbors = uv_player_rtree
            .0
            .nearest_neighbor_iterator(&pos.0)
            .skip(1)
            .take(5)
            .collect();

        // When evaluating the behaviour of the agent, some simple context is set up
        let ctx = BehaviourContext {
            current_player: (pos, vel),
            distance_to_it: 0.0,
            nearest_5_neighbors,
        };

        // Behaviours dictate how the players act - mostly their orientation
        match tag {
            TagState::NotIt => behaviour.not_it_behaviour.revise_orientation(ctx),
            TagState::It => behaviour.it_behaviour.revise_orientation(ctx),
        };
    }
}

fn tag_collided_players(
    v_position: View<Position>,
    uv_time: UniqueView<Time>,
    mut vm_recently_tagged: ViewMut<RecentlyTagged>,
    mut vm_tagged: ViewMut<Tagged>,
    //`mut recently_tagged: UniqueViewMut<RecentlyTagged>,
) {
    // The execution time characteristics of the following is _not ideal_.
    // Collisions should ideally be handled through specialized data structures.
    // For example quad trees.

    // The currently, and only, tagged player
    let tagged_it = (&v_position, &vm_tagged)
        .iter()
        .with_id()
        .find(|(_, (_, tag))| tag.0 == TagState::It)
        .clone();

    if let Some((it_id, (it_pos, _))) = tagged_it {
        let mut have_tagged_new = false;

        for (pos, mut tag, mut recently_tagged) in
            (&v_position, &mut vm_tagged, &mut vm_recently_tagged).iter()
        {
            // The tagged player is touching another, non-tagged player
            if tag.0 != TagState::It
                && (&it_pos).distance_to(&pos) <= PLAYER_SIZE * 2.0
                && recently_tagged.0.is_none()
            {
                // Untag the player who is currently "it", tag the "non-it" player and stop.
                tag.0 = TagState::It;
                have_tagged_new = true;

                // mark newly-tagged player as recently tagged
                recently_tagged.0 = Some(uv_time.0);
                break;
            }
        }

        // If the tagging has been replaced, update the now previously tagged player.
        if have_tagged_new {
            if let Ok((mut recently_tagged, mut tagged)) =
                (&mut vm_recently_tagged, &mut vm_tagged).get(it_id)
            {
                tagged.0 = TagState::NotIt;
                // mark previously-tagged player as recently tagged
                recently_tagged.0 = Some(uv_time.0);
            };
        }
    }
}

// 5 seconds after tagging, players will be removed from the
// recently-tagged players map
const CLEAR_AFTER_SECONDS: u128 = 5 * 1000; // milliseconds
fn clear_old_recently_tagged(
    uv_time: UniqueView<Time>,
    mut vm_recently_tagged: ViewMut<RecentlyTagged>,
    // mut recently_tagged: UniqueViewMut<RecentlyTagged>,
) {
    // Only keep recent players if their timestamp is within the given constant
    (&mut vm_recently_tagged).iter().for_each(|t| {
        if let Some(timestamp) = t.0 {
            if timestamp + CLEAR_AFTER_SECONDS <= uv_time.0 {
                t.0.take();
            }
        }
    });
    // for (mut rt) in (&mut vm_recently_tagged).iter().filter(|t| t.0.is_some()) {}
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
        let tag = &tag.0;

        // The drawn direction vector is shown relative to the drawn player size.
        let direction_vector = (vel.clone() * PLAYER_SIZE).0;

        // Players that are "it" will have a different color.
        let color = match tag {
            TagState::NotIt => Color::BLACK,
            TagState::It => Color::GOLD,
        };
        d.draw_circle(pos.0[0] as i32, pos.0[1] as i32, PLAYER_SIZE, color);
        let ray_pos: Vector2 = pos.into();
        let ray_end_pos: Vector2 = Vector2 {
            x: ray_pos.x + (direction_vector[0]),
            y: ray_pos.y + (direction_vector[1]),
        };
        d.draw_line_ex(
            ray_pos,
            ray_end_pos,
            f32::max(PLAYER_SIZE * 0.5, 1.0),
            Color::BLUE,
        )
    }
}
