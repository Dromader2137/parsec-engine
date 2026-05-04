//! Integer vector types.

use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct Vec2i {
    pub x: i32,
    pub y: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct Vec3i {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl Add for Vec2i {
    type Output = Vec2i;
    fn add(self, rhs: Self) -> Self::Output {
        Vec2i::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Vec2i {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Add for Vec3i {
    type Output = Vec3i;
    fn add(self, rhs: Self) -> Self::Output {
        Vec3i::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign for Vec3i {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Div for Vec2i {
    type Output = Vec2i;
    fn div(self, rhs: Self) -> Self::Output {
        Vec2i::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl Div<i32> for Vec2i {
    type Output = Vec2i;
    fn div(self, rhs: i32) -> Self::Output {
        Vec2i::new(self.x / rhs, self.y / rhs)
    }
}

impl DivAssign for Vec2i {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl DivAssign<i32> for Vec2i {
    fn div_assign(&mut self, rhs: i32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Div for Vec3i {
    type Output = Vec3i;
    fn div(self, rhs: Self) -> Self::Output {
        Vec3i::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}

impl Div<i32> for Vec3i {
    type Output = Vec3i;
    fn div(self, rhs: i32) -> Self::Output {
        Vec3i::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl DivAssign for Vec3i {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

impl DivAssign<i32> for Vec3i {
    fn div_assign(&mut self, rhs: i32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Mul<i32> for Vec2i {
    type Output = Vec2i;
    fn mul(self, rhs: i32) -> Self::Output {
        Vec2i::new(self.x * rhs, self.y * rhs)
    }
}

impl MulAssign for Vec2i {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl MulAssign<i32> for Vec2i {
    fn mul_assign(&mut self, rhs: i32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Mul for Vec3i {
    type Output = Vec3i;
    fn mul(self, rhs: Self) -> Self::Output {
        Vec3i::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl Mul<i32> for Vec3i {
    type Output = Vec3i;
    fn mul(self, rhs: i32) -> Self::Output {
        Vec3i::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl MulAssign for Vec3i {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl MulAssign<i32> for Vec3i {
    fn mul_assign(&mut self, rhs: i32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Sub for Vec2i {
    type Output = Vec2i;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec2i::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Vec2i {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Sub for Vec3i {
    type Output = Vec3i;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec3i::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl SubAssign for Vec3i {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Vec2i {
    pub const ZERO: Vec2i = Vec2i { x: 0, y: 0 };
    pub const ONE: Vec2i = Vec2i { x: 1, y: 1 };
    pub const UP: Vec2i = Vec2i { x: 0, y: 1 };
    pub const DOWN: Vec2i = Vec2i { x: 0, y: -1 };
    pub const LEFT: Vec2i = Vec2i { x: 1, y: 0 };
    pub const RIGHT: Vec2i = Vec2i { x: -1, y: 0 };

    pub fn new(x: i32, y: i32) -> Vec2i { Vec2i { x, y } }

    pub fn dot(&mut self, vec: Vec2i) -> i32 { self.x * vec.x + self.y * vec.y }

    pub fn cross(&mut self, vec: Vec2i) -> i32 {
        (self.x * vec.y) - (self.y * vec.x)
    }

    pub fn length_sqr(&mut self) -> i32 { self.x * self.x + self.y * self.y }
}

impl Vec3i {
    pub const ZERO: Vec3i = Vec3i { x: 0, y: 0, z: 0 };
    pub const ONE: Vec3i = Vec3i { x: 1, y: 1, z: 1 };
    pub const UP: Vec3i = Vec3i { x: 0, y: 1, z: 0 };
    pub const DOWN: Vec3i = Vec3i { x: 0, y: -1, z: 0 };
    pub const LEFT: Vec3i = Vec3i { x: 1, y: 0, z: 0 };
    pub const RIGHT: Vec3i = Vec3i { x: -1, y: 0, z: 0 };
    pub const FORWARD: Vec3i = Vec3i { x: 0, y: 0, z: 1 };
    pub const BACK: Vec3i = Vec3i { x: 0, y: 0, z: -1 };

    pub fn new(x: i32, y: i32, z: i32) -> Vec3i { Vec3i { x, y, z } }

    pub fn dot(&mut self, vec: Vec3i) -> i32 {
        self.x * vec.x + self.y * vec.y + self.z * vec.z
    }

    pub fn cross(&mut self, vec: Vec3i) -> Vec3i {
        Vec3i {
            x: (self.y * vec.z) - (self.z * vec.y),
            y: (self.z * vec.x) - (self.x * vec.z),
            z: (self.x * vec.y) - (self.y * vec.x),
        }
    }

    pub fn length_sqr(&mut self) -> i32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
}
