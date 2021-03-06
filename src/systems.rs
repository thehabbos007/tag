use std::borrow::BorrowMut;

use rand::Rng;
use raylib::prelude::*;
use shipyard::*;
use spade::rtree::RTree;

use crate::{
    behaviours::{BehaviourAction, BehaviourContext},
    entities_components::{
        PlayerBehaviour, PlayersPositionRTree, Position, RLHandle, RLThread, RTreeData,
        RecentlyTagged, TagCount, TagState, Tagged, Time, Velocity,
    },
    Endurance, HEIGHT, PLAYER_SIZE, WIDTH,
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

pub const TICK: &str = "TICK";
pub const FRAME: &str = "FRAME";

/// A collections of systems to run for each frame.
pub fn register_workloads(world: &World) {
    Workload::builder(TICK)
        // Move players given their velocity every tick
        .with_system(update_player_position)
        .with_system(update_player_position_rtee)
        .with_system(commit_player_behaviour)
        // Play tag
        .with_system(tag_collided_players)
        // Clear recently tagged players
        .with_system(clear_old_recently_tagged)
        .with_system(regain_endurance)
        .add_to_world(&world)
        .unwrap();

    Workload::builder(FRAME)
        // Render the players at the end of the frame tick
        .with_system(render_players)
        .add_to_world(&world)
        .unwrap();
}

/// Move players in accordance to their velocity
fn update_player_position(
    v_velocity: View<Velocity>,
    mut vm_endurance: ViewMut<Endurance>,
    mut vm_position: ViewMut<Position>,
) {
    for (pos, endurance, vel) in (&mut vm_position, &mut vm_endurance, &v_velocity).iter() {
        let geo_pos = &mut pos.0;
        let geo_vel = endurance_velocity_scale(endurance, vel).0;

        // A player will wrap around if they hit corners 2d-game style
        geo_pos[0] = (geo_pos[0] + (geo_vel[0])).rem_euclid(WIDTH as f32);
        geo_pos[1] = (geo_pos[1] + (geo_vel[1])).rem_euclid(HEIGHT as f32);

        endurance.0 = u16::max(endurance.0 - 1, 1);
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
            .with_id()
            .map(|(entity_id, (pos, vel, recently_tagged, tag))| RTreeData {
                entity_id,
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

/// Players have their own behaviour which differs between when they're "it" and "not it"
/// A player might wander aimlessly when "not it" but target nearest neighbours when "it"
fn regain_endurance(v_tagged: View<Tagged>, mut vm_endurance: ViewMut<Endurance>) {
    let mut rng = rand::thread_rng();

    for (tag, endurance) in (&v_tagged, &mut vm_endurance).iter() {
        match tag.0 {
            TagState::NotIt => {
                if rng.gen_bool(0.01) && endurance.0 < endurance.1 {
                    endurance.0 += rng.gen_range(0..50);
                }
            }
            TagState::It => {
                if rng.gen_bool(0.05) && endurance.0 < endurance.1 {
                    endurance.0 += rng.gen_range(0..50);
                }
            }
        }
    }
}

/// Tag players that collide with "it" players.
/// Using the [PlayersPositionRTree] it is possible to do this with good performance.
fn tag_collided_players(
    v_position: View<Position>,
    uv_time: UniqueView<Time>,
    uv_player_rtree: UniqueView<PlayersPositionRTree>,
    mut uvm_tag_count: UniqueViewMut<TagCount>,
    mut vm_recently_tagged: ViewMut<RecentlyTagged>,
    mut vm_tagged: ViewMut<Tagged>,
) {
    // The currently, and only, tagged player
    let tagged_it = (&v_position, &vm_tagged)
        .iter()
        .with_id()
        .find(|(_, (_, tag))| tag.0 == TagState::It)
        .clone();

    let mut have_tagged_new = false;
    if let Some((it_id, (it_pos, _))) = tagged_it {
        let closest = uv_player_rtree.0.nearest_n_neighbors(&it_pos.0, 2);
        // skip the first, as that would be the current "it" player
        if let Some(RTreeData { entity_id, .. }) = closest.get(1) {
            if let Ok((position, mut tagged, mut recently_tagged)) =
                (&v_position, &mut vm_tagged, &mut vm_recently_tagged).get(entity_id.clone())
            {
                if tagged.0 != TagState::It
                    && recently_tagged.0.is_none()
                    && it_pos.distance_to(position) <= PLAYER_SIZE * 2.0
                {
                    have_tagged_new = true;
                    tagged.0 = TagState::It;

                    recently_tagged.0 = Some(uv_time.0);
                    uvm_tag_count.0 += 1;
                }
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

                // Increment total tag count
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
}

/// Render player positions into the raylib framebuffer.
fn render_players(
    mut rlh: UniqueViewMut<RLHandle>,
    rlt: NonSendSync<UniqueView<RLThread>>,
    positions: View<Position>,
    velocities: View<Velocity>,
    endurance: View<Endurance>,
    tagged: View<Tagged>,
    uv_tag_count: UniqueView<TagCount>,
) {
    let mut d = rlh.0.begin_drawing(&rlt.0);

    d.clear_background(Color::WHITE);

    // Each player is shown with position as dot, line as direction they're facing
    for (pos, vel, endurance, tag) in (&positions, &velocities, &endurance, &tagged).iter() {
        let tag = &tag.0;

        let vel = endurance_velocity_scale(endurance, vel);

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

    d.draw_text(
        format!("Total taggings: {}", uv_tag_count.0).as_str(),
        12,
        12,
        20,
        Color::DARKPURPLE,
    );
}

/// Helper function to calculated endurace-scaled velocity
fn endurance_velocity_scale(endurance: &Endurance, vel: &Velocity) -> Velocity {
    let Endurance(current_endurance, max_endurance) = *endurance;
    let endurance_factor = 1.0 - current_endurance as f32 / max_endurance as f32;
    vel.clone() - (vel.clone() / 2. * endurance_factor)
}
