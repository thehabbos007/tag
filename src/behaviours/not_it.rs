use rand::Rng;

use super::{BehaviourAction, BehaviourContext};

/// When an actor is "not it" they can behave in these states.
#[derive(Debug)]
pub enum NotItBehaviour {
    Random(RandomBehaviour),
    OpposeIt(OpposeItBehaviour),
}

impl Default for NotItBehaviour {
    fn default() -> Self {
        NotItBehaviour::Random(RandomBehaviour)
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
        let [x, y] = &mut ctx.current_player.1 .0;

        // Every now and then the player's direction changes
        if rng.gen_bool(0.005) {
            *x = (*x + rng.gen_range(-1..1)) % 5;
            *y = (*y + rng.gen_range(-1..1)) % 5;
        }
    }
}

/// The "not it" players with this behaviour will be averse to the currently tagged actor.
#[derive(Debug)]
pub struct OpposeItBehaviour;

impl BehaviourAction for OpposeItBehaviour {
    fn revise_orientation(&self, _ctx: BehaviourContext) {
        // let (current_pos, current_vel) = ctx.current_player;
    }
}
