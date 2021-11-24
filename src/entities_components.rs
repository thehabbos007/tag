use rand::distributions::{Distribution, Standard};
use rand::Rng;
use shipyard::Component;

// Point and vec are not generic for now. The main reason is that it's "simple" enough so far.
// If more overlap appears, these could be combined - there is some duplication currently.
// Generics with numeric operations isn't a breeze either.

/// A geometric object that has `x` and `y` components
#[derive(Debug, Clone)]
pub struct Point2D {
    pub x: u16,
    pub y: u16,
}

impl Point2D {
    pub fn new(x: u16, y: u16) -> Self {
        Self { x, y }
    }
    /// restrict the point's `point.x` and `point.y` values to be at max [x] and [y]
    pub fn within(mut self, x: u16, y: u16) -> Self {
        self.x %= x;
        self.y %= y;
        self
    }
}

impl Distribution<Point2D> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Point2D {
        Point2D {
            x: rng.gen(),
            y: rng.gen(),
        }
    }
}

/// A geometric 2d vector that has `x` and `y` components
#[derive(Debug, Clone)]
pub struct Vec2d {
    pub x: f32,
    pub y: f32,
}
impl Vec2d {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

impl Distribution<Vec2d> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Vec2d {
        Vec2d {
            x: rng.gen(),
            y: rng.gen(),
        }
    }
}

// Components

/// Position is a conventional variant of a point, it denotes a location in space
#[derive(Debug, Component)]
pub struct Position(pub Point2D);

/// Velocity is a vector that has a direction and a magnitude.
/// Direction models direction of the player, magnitude models speed.
#[derive(Clone, Debug, Component)]
pub struct Velocity(pub Vec2d);

impl Velocity {
    /// Get the magnitude (length) of the vector - the speed of the entity
    #[allow(dead_code)]
    pub fn magnitude(&self) -> f32 {
        let fx = self.0.x as f32;
        let fy = self.0.y as f32;
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
        Velocity(Vec2d {
            x: self.0.x * value,
            y: self.0.y * value,
        })
    }
}

// Wrap raylib structs in a Shipyard ECS component.
#[derive(Component)]
pub struct RLHandle(pub raylib::RaylibHandle);
#[derive(Component)]
pub struct RLThread(pub raylib::RaylibThread);
