use rand::prelude::*;
use shipyard::*;

mod entities_components;
use crate::entities_components::*;

/// Number of players in `world`
const PLAYER_COUNT: usize = 5;
/// Width of the `world` view
const WIDTH: u16 = 1024;
/// Height of the `world` view
const HEIGHT: u16 = 1024;
fn main() {
    let mut world = World::default();
    let mut rng = rand::thread_rng();

    world
        .bulk_add_entity((0..PLAYER_COUNT).map(|_| {
            (
                Position(rng.gen::<Point2D>().within(WIDTH, HEIGHT)),
                Velocity(Vec2d::new(0.0, 1.0)),
            )
        }))
        .next();

    let (pos, vels) = world.borrow::<(View<Position>, View<Velocity>)>().unwrap();

    for (id, (p, v)) in (&pos, &vels).iter().with_id() {
        println!("({:?}, {:?}) belongs to players {:?}", p, v, id);
    }
}
