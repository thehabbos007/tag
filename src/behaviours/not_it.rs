use rand::Rng;

use crate::entities_components::TagState;

use super::{BehaviourAction, BehaviourContext};

/// When an actor is "not it" they can behave in these states.
#[derive(Debug)]
pub enum NotItBehaviour {
    OpposeIt(OpposeItBehaviour),
    Random(RandomBehaviour),
}

impl Default for NotItBehaviour {
    fn default() -> Self {
        NotItBehaviour::OpposeIt(OpposeItBehaviour)
    }
}

impl BehaviourAction for NotItBehaviour {
    fn revise_orientation(&self, ctx: BehaviourContext) {
        match self {
            NotItBehaviour::OpposeIt(b) => b.revise_orientation(ctx),
            NotItBehaviour::Random(b) => b.revise_orientation(ctx),
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
        if rng.gen_bool(0.02) {
            *vel = vel.rotate_angle(rng.gen_range(0.0..std::f32::consts::PI))
        }
    }
}

/// The "not it" players with this behaviour will be averse to the currently tagged actor.
#[derive(Debug)]
pub struct OpposeItBehaviour;

impl BehaviourAction for OpposeItBehaviour {
    fn revise_orientation(&self, ctx: BehaviourContext) {
        if let Some(near) = ctx
            .nearest_5_neighbors
            .iter()
            .find(|n| n.tagged.0 == TagState::It)
        {
            let my_pos = ctx.current_player.0;
            if my_pos.distance_to(&near.position) > 0.5 {
                let my_vel = ctx.current_player.1;
                let near_pos = &near.position;

                let v_target = my_pos.velocity_facing(near_pos);
                let angle = my_vel.angle_between(&v_target);

                let new_vel = my_vel.rotate_angle(angle).negate_velocity();
                *my_vel = new_vel;
            }
        } else {
            RandomBehaviour::revise_orientation(&RandomBehaviour, ctx);
        }
    }
}
