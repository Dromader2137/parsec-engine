//! Unsigned integer vector types.

use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct Vec2u {
    pub x: u32,
    pub y: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct Vec3u {
    pub x: u32,
    pub y: u32,
    pub z: u32,
}

impl Add for Vec2u {
    type Output = Vec2u;
    fn add(self, rhs: Self) -> Self::Output {
        Vec2u::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Vec2u {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Add for Vec3u {
    type Output = Vec3u;
    fn add(self, rhs: Self) -> Self::Output {
        Vec3u::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign for Vec3u {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Div for Vec2u {
    type Output = Vec2u;
    fn div(self, rhs: Self) -> Self::Output {
        Vec2u::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl Div<u32> for Vec2u {
    type Output = Vec2u;
    fn div(self, rhs: u32) -> Self::Output {
        Vec2u::new(self.x / rhs, self.y / rhs)
    }
}

impl DivAssign for Vec2u {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl DivAssign<u32> for Vec2u {
    fn div_assign(&mut self, rhs: u32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Div for Vec3u {
    type Output = Vec3u;
    fn div(self, rhs: Self) -> Self::Output {
        Vec3u::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}

impl Div<u32> for Vec3u {
    type Output = Vec3u;
    fn div(self, rhs: u32) -> Self::Output {
        Vec3u::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl DivAssign for Vec3u {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

impl DivAssign<u32> for Vec3u {
    fn div_assign(&mut self, rhs: u32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Mul<u32> for Vec2u {
    type Output = Vec2u;
    fn mul(self, rhs: u32) -> Self::Output {
        Vec2u::new(self.x * rhs, self.y * rhs)
    }
}

impl MulAssign for Vec2u {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl MulAssign<u32> for Vec2u {
    fn mul_assign(&mut self, rhs: u32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Mul for Vec3u {
    type Output = Vec3u;
    fn mul(self, rhs: Self) -> Self::Output {
        Vec3u::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl Mul<u32> for Vec3u {
    type Output = Vec3u;
    fn mul(self, rhs: u32) -> Self::Output {
        Vec3u::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl MulAssign for Vec3u {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl MulAssign<u32> for Vec3u {
    fn mul_assign(&mut self, rhs: u32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Sub for Vec2u {
    type Output = Vec2u;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec2u::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Vec2u {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Sub for Vec3u {
    type Output = Vec3u;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec3u::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl SubAssign for Vec3u {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Vec2u {
    pub const ZERO: Vec2u = Vec2u { x: 0, y: 0 };
    pub const ONE: Vec2u = Vec2u { x: 1, y: 1 };
    pub const UP: Vec2u = Vec2u { x: 0, y: 1 };
    pub const LEFT: Vec2u = Vec2u { x: 1, y: 0 };

    pub fn new(x: u32, y: u32) -> Vec2u { Vec2u { x, y } }

    pub fn dot(&mut self, vec: Vec2u) -> u32 { self.x * vec.x + self.y * vec.y }

    pub fn cross(&mut self, vec: Vec2u) -> u32 {
        (self.x * vec.y) - (self.y * vec.x)
    }

    pub fn length_sqr(&mut self) -> u32 { self.x * self.x + self.y * self.y }
}

impl Vec3u {
    pub const ZERO: Vec3u = Vec3u { x: 0, y: 0, z: 0 };
    pub const ONE: Vec3u = Vec3u { x: 1, y: 1, z: 1 };
    pub const UP: Vec3u = Vec3u { x: 0, y: 1, z: 0 };
    pub const LEFT: Vec3u = Vec3u { x: 1, y: 0, z: 0 };
    pub const FORWARD: Vec3u = Vec3u { x: 0, y: 0, z: 1 };

    pub fn new(x: u32, y: u32, z: u32) -> Vec3u { Vec3u { x, y, z } }

    pub fn dot(&mut self, vec: Vec3u) -> u32 {
        self.x * vec.x + self.y * vec.y + self.z * vec.z
    }

    pub fn cross(&mut self, vec: Vec3u) -> Vec3u {
        Vec3u {
            x: (self.y * vec.z) - (self.z * vec.y),
            y: (self.z * vec.x) - (self.x * vec.z),
            z: (self.x * vec.y) - (self.y * vec.x),
        }
    }

    pub fn length_sqr(&mut self) -> u32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }
}
