use std::collections::HashMap;

use rand::distributions::{Distribution, Standard};
use rand::Rng;
use raylib::math::Vector2;
use shipyard::{Component, EntityId};

use crate::behaviours::{ItBehaviour, NotItBehaviour};
use crate::{HEIGHT, WIDTH};

/// A geometric object that has `x` and `y` components

pub type Geo2D = [i32; 2];

/// A player can either "be it" or "not be it"
#[derive(PartialEq, Eq, Debug)]
pub enum TagState {
    NotIt,
    It,
}

impl Default for TagState {
    fn default() -> Self {
        Self::NotIt
    }
}

// Components

/// Position is a conventional variant of a point, it denotes a location in space
#[derive(Debug, Component)]
pub struct Position(pub Geo2D);

impl Distribution<Position> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Position {
        Position::new(rng.gen_range(0..WIDTH), rng.gen_range(0..HEIGHT))
    }
}
impl Into<Vector2> for &Position {
    fn into(self) -> Vector2 {
        Vector2 {
            x: self.0[0] as f32,
            y: self.0[1] as f32,
        }
    }
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self([x, y])
    }

    pub fn distance_to(&self, other: &Position) -> f32 {
        let [x1, y1] = self.0;
        let [x2, y2] = other.0;
        f32::sqrt(i32::pow(x2 - x1, 2) as f32 + i32::pow(y2 - y1, 2) as f32)
    }
}

/// Velocity is a vector that has a direction and a magnitude.
/// Direction models direction of the player, magnitude models speed.
#[derive(Clone, Debug, Component)]
pub struct Velocity(pub Geo2D);

impl Distribution<Velocity> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Velocity {
        Velocity::new(rng.gen_range(-3..3), rng.gen_range(-3..3))
    }
}

impl Velocity {
    pub fn new(x: i32, y: i32) -> Self {
        Self([x, y])
    }
    /// restrict the point's `point.x` and `point.y` values to be at max [x] and [y]
    pub fn within(mut self, x: i32, y: i32) -> Self {
        self.0[0] %= x;
        self.0[1] %= y;
        self
    }

    /// Get the magnitude (length) of the vector - the speed of the entity
    #[allow(dead_code)]
    pub fn magnitude(&self) -> f32 {
        let fx = self.0[0] as f32;
        let fy = self.0[1] as f32;
        f32::sqrt(fx * fx + fy * fy)
    }

    /// Quake-style fast inverse square root use for normalizing vectors
    /// https://stackoverflow.com/a/59083859
    fn inv_sqrt(&self) -> f32 {
        let x = self.magnitude();
        let i = x.to_bits();
        let i = 0x5f3759df - (i >> 1);
        let y = f32::from_bits(i);

        y * (1.5 - 0.5 * x * y * y)
    }

    /// Normalize given vector, get new vector from that.
    pub fn normalize(&self) -> Self {
        let factor = self.inv_sqrt();
        self.clone() * factor
    }
}

impl std::ops::Mul<f32> for Velocity {
    type Output = Velocity;
    fn mul(self, value: f32) -> Velocity {
        let [x, y] = self.0;
        Velocity::new(x * value as i32, y * value as i32)
    }
}

/// Is player "it" or "not it"
#[derive(Debug, Component)]
pub struct Tagged(pub TagState);

/// How does the player behave

#[derive(Default, Debug, Component)]
pub struct PlayerBehaviour {
    pub it_behaviour: ItBehaviour,
    pub not_it_behaviour: NotItBehaviour,
}

/// Epoch timestamp in milliseconds
#[derive(Component)]
pub struct Time(pub u128);

/// Map of recently tagged players
#[derive(Component)]
pub struct RecentlyTagged(pub HashMap<EntityId, u128>);

/// Wrap raylib handler in a Shipyard ECS component.
#[derive(Component)]
pub struct RLHandle(pub raylib::RaylibHandle);

/// Wrap raylib thread in a Shipyard ECS component.
#[derive(Component)]
pub struct RLThread(pub raylib::RaylibThread);
