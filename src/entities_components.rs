use rand::distributions::{Distribution, Standard};
use rand::Rng;
use raylib::math::Vector2;
use shipyard::Component;

use crate::{HEIGHT, WIDTH};

/// A geometric object that has `x` and `y` components
#[derive(Debug, Clone)]
pub struct Geo2D {
    pub x: i16,
    pub y: i16,
}

impl Into<Vector2> for &Geo2D {
    fn into(self) -> Vector2 {
        Vector2 {
            x: self.x as f32,
            y: self.y as f32,
        }
    }
}

impl Geo2D {
    pub fn new(x: i16, y: i16) -> Self {
        Self { x, y }
    }
    /// restrict the point's `point.x` and `point.y` values to be at max [x] and [y]
    pub fn within(mut self, x: i16, y: i16) -> Self {
        self.x %= x;
        self.y %= y;
        self
    }

    /// Get the magnitude (length) of the vector - the speed of the entity
    #[allow(dead_code)]
    pub fn magnitude(&self) -> f32 {
        let fx = self.x as f32;
        let fy = self.y as f32;
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

impl std::ops::Mul<f32> for Geo2D {
    type Output = Geo2D;
    fn mul(self, value: f32) -> Geo2D {
        Geo2D {
            x: self.x * (value as i16),
            y: self.y * (value as i16),
        }
    }
}

// Components

/// Position is a conventional variant of a point, it denotes a location in space
#[derive(Debug, Component)]
pub struct Position(pub Geo2D);

impl Distribution<Position> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Position {
        Position(Geo2D {
            x: rng.gen_range(0..WIDTH),
            y: rng.gen_range(0..HEIGHT),
        })
    }
}

/// Velocity is a vector that has a direction and a magnitude.
/// Direction models direction of the player, magnitude models speed.
#[derive(Debug, Component)]
pub struct Velocity(pub Geo2D);

impl Distribution<Velocity> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Velocity {
        Velocity(Geo2D {
            x: rng.gen_range(-3..3),
            y: rng.gen_range(-3..3),
        })
    }
}

// Wrap raylib structs in a Shipyard ECS component.
#[derive(Component)]
pub struct RLHandle(pub raylib::RaylibHandle);
#[derive(Component)]
pub struct RLThread(pub raylib::RaylibThread);
