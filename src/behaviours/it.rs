use rand::Rng;

use super::{BehaviourAction, BehaviourContext};

/// When an actor is "it" they can behave in these states.
#[derive(Debug)]
pub enum ItBehaviour {
    RandomBehaviour(RandomBehaviour),
    ChaseNearest(ChaseNearestBehaviour),
}

impl Default for ItBehaviour {
    fn default() -> Self {
        ItBehaviour::ChaseNearest(ChaseNearestBehaviour)
    }
}

impl BehaviourAction for ItBehaviour {
    fn revise_orientation(&self, ctx: BehaviourContext) {
        match self {
            ItBehaviour::ChaseNearest(b) => b.revise_orientation(ctx),
            ItBehaviour::RandomBehaviour(b) => b.revise_orientation(ctx),
        };
    }
}

/// Sometimes the player will adjust it's orientation randomly with this behaviour.
#[derive(Debug)]
pub struct RandomBehaviour;

impl BehaviourAction for RandomBehaviour {
    fn revise_orientation(&self, ctx: BehaviourContext) {
        let mut rng = rand::thread_rng();
        let vel = ctx.current_player.1;

        // Every now and then the player's direction changes
        if rng.gen_bool(0.005) {
            *vel = vel.rotate_angle(rng.gen_range(0.0..std::f32::consts::TAU))
        }
    }
}

/// The tagged "it" player will try to "cut corners" and predict movement of its closest neighbour.
#[derive(Debug)]
pub struct ChaseNearestBehaviour;

impl BehaviourAction for ChaseNearestBehaviour {
    fn revise_orientation(&self, ctx: BehaviourContext) {
        if let Some(near) = ctx.nearest_5_neighbors.iter().find(|n| !n.recently_tagged) {
            let my_pos = ctx.current_player.0;
            if my_pos.distance_to(&near.position) > 0.5 {
                let my_vel = ctx.current_player.1;
                let near_pos = &near.position;

                let v_target = my_pos.velocity_facing(near_pos);
                let angle = my_vel.angle_between(&v_target);

                let new_vel = my_vel.rotate_angle(angle);
                *my_vel = new_vel;
            }
        } else {
            RandomBehaviour::revise_orientation(&RandomBehaviour, ctx);
        }
    }
}
