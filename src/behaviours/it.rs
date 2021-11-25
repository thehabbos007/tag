use rand::Rng;

use super::{BehaviourAction, BehaviourContext};

/// When an actor is "it" they can behave in these states.
#[derive(Debug)]
pub enum ItBehaviour {
    ChaseNearest(TriangulateNearestBehaviour),
    RandomBehaviour(RandomBehaviour),
}

impl Default for ItBehaviour {
    fn default() -> Self {
        ItBehaviour::ChaseNearest(TriangulateNearestBehaviour)
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
        let [x, y] = &mut ctx.current_player.1 .0;

        // Every now and then the player's direction changes
        if rng.gen_bool(0.005) {
            *x = (*x + rng.gen_range(-1..1)) % 5;
            *y = (*y + rng.gen_range(-1..1)) % 5;
        }
    }
}

/// The tagged "it" player will try to "cut corners" and predict movement of its closest neighbour.
#[derive(Debug)]
pub struct TriangulateNearestBehaviour;

impl BehaviourAction for TriangulateNearestBehaviour {
    fn revise_orientation(&self, _ctx: BehaviourContext) {
        // let (_it_pos, it_vel) = ctx.current_player;
    }
}
