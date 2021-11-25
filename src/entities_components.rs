use std::collections::HashMap;

use rand::distributions::{Distribution, Standard};
use rand::Rng;
use raylib::math::Vector2;
use shipyard::{Component, EntityId};
use spade::rtree::RTree;

use crate::behaviours::{ItBehaviour, NotItBehaviour};
use crate::{HEIGHT, WIDTH};

/// A geometric object that has `x` and `y` components

pub type Geo2D = [f32; 2];

/// A player can either "be it" or "not be it"
#[derive(Clone, PartialEq, Eq, Debug)]
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
#[derive(PartialEq, Clone, Debug, Component)]
pub struct Position(pub Geo2D);

impl Distribution<Position> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Position {
        Position::new(
            rng.gen_range(0.0..(WIDTH as f32)),
            rng.gen_range(0.0..(HEIGHT as f32)),
        )
    }
}
impl Into<Vector2> for &Position {
    fn into(self) -> Vector2 {
        Vector2 {
            x: self.0[0],
            y: self.0[1],
        }
    }
}

impl Position {
    pub fn new(x: f32, y: f32) -> Self {
        Self([x, y])
    }

    pub fn distance_to(&self, other: &Position) -> f32 {
        let [x1, y1] = self.0;
        let [x2, y2] = other.0;
        f32::sqrt(f32::powi(x2 - x1, 2) + f32::powi(y2 - y1, 2))
    }
}

/// Velocity is a vector that has a direction and a magnitude.
/// Direction models direction of the player, magnitude models speed.
#[derive(PartialEq, Clone, Debug, Component)]
pub struct Velocity(pub Geo2D);

impl Distribution<Velocity> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Velocity {
        Velocity::new(rng.gen_range(-3.0..3.0), rng.gen_range(-3.0..3.0))
    }
}

impl Velocity {
    pub fn new(x: f32, y: f32) -> Self {
        Self([x, y])
    }
    /// restrict the point's `point.x` and `point.y` values to be at max [x] and [y]
    pub fn within(mut self, x: f32, y: f32) -> Self {
        self.0[0] %= x;
        self.0[1] %= y;
        self
    }

    /// Get the magnitude (length) of the vector - the speed of the entity
    #[allow(dead_code)]
    pub fn magnitude(&self) -> f32 {
        let fx = self.0[0];
        let fy = self.0[1];
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

    pub fn rotate_towards(&self, other: &Geo2D) -> Self {
        let [x1, y1] = [self.0[0], self.0[1]];
        let [x2, y2] = [other[0], other[1]];

        let dot = x1 * x2 + y1 * y2;
        let det = x1 * y2 - y1 * x2;
        let angle = f32::atan2(det, dot);

        let x = x1 * f32::cos(angle) - y1 * f32::sin(angle);
        let y = x1 * f32::sin(angle) + y1 * f32::cos(angle);

        Self::new(x, y)
    }
}

/// Scalar multiplication of velocity
impl std::ops::Mul<f32> for Velocity {
    type Output = Velocity;
    fn mul(self, value: f32) -> Velocity {
        let [x, y] = self.0;
        Velocity::new(x * value, y * value)
    }
}

/// Scalar division of velocity
impl std::ops::Div<f32> for Velocity {
    type Output = Velocity;
    fn div(self, value: f32) -> Velocity {
        let [x, y] = self.0;
        Velocity::new(x / value, y / value)
    }
}

/// Is player "it" or "not it"
#[derive(Clone, Default, Debug, Component)]
pub struct Tagged(pub TagState);

/// How does the player behave

#[derive(Default, Debug, Component)]
pub struct PlayerBehaviour {
    pub it_behaviour: ItBehaviour,
    pub not_it_behaviour: NotItBehaviour,
}

/// Epoch timestamp in milliseconds
#[derive(Default, Component)]
pub struct Time(pub u128);

/// Map of recently tagged players
#[derive(Default, Component)]
pub struct RecentlyTagged(pub Option<u128>);

/// Map of recently tagged players
#[derive(Default, Component)]
pub struct PlayersPositionRTree(pub RTree<RTreeData>);

#[derive(Clone, Debug)]
pub struct RTreeData {
    pub position: Position,
    pub velocity: Velocity,
    pub recently_tagged: bool,
    pub tagged: Tagged,
}
impl spade::SpatialObject for RTreeData {
    type Point = Geo2D;

    fn mbr(&self) -> spade::BoundingRect<Self::Point> {
        spade::BoundingRect::from_point(self.position.0)
    }

    fn distance2(&self, point: &Self::Point) -> <Self::Point as spade::PointN>::Scalar {
        self.position.0.distance2(point)
    }
}

/// Wrap raylib handler in a Shipyard ECS component.
#[derive(Component)]
pub struct RLHandle(pub raylib::RaylibHandle);

/// Wrap raylib thread in a Shipyard ECS component.
#[derive(Component)]
pub struct RLThread(pub raylib::RaylibThread);

#[cfg(test)]
mod test {
    use super::Velocity;
    macro_rules! assert_delta {
        ($x:expr, $y:expr, $d:expr) => {
            if !($x - $y < $d || $y - $x < $d) {
                panic!();
            }
        };
    }

    #[test]
    fn test_rotation() {
        let v1 = Velocity::new(1.0, 1.0);
        let v2 = Velocity::new(-7.0, 7.0);
        let rotated = v1.rotate_towards(&v2.0).0;

        assert_delta!(rotated[0], -1.0, 0.001);
        assert_delta!(rotated[1], 1.0, 0.001);
    }
}
