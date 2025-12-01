//! Vector types.

use std::ops::{
    Add, AddAssign, Div, DivAssign, Mul, MulAssign, Sub, SubAssign,
};

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Vec2f {
    pub x: f32,
    pub y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Vec3f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Vec4f {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub w: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Vec2d {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Vec3d {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Add for Vec2f {
    type Output = Vec2f;
    fn add(self, rhs: Self) -> Self::Output {
        Vec2f::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Vec2f {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Add for Vec3f {
    type Output = Vec3f;
    fn add(self, rhs: Self) -> Self::Output {
        Vec3f::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign for Vec3f {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Add for Vec2d {
    type Output = Vec2d;
    fn add(self, rhs: Self) -> Self::Output {
        Vec2d::new(self.x + rhs.x, self.y + rhs.y)
    }
}

impl AddAssign for Vec2d {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl Add for Vec3d {
    type Output = Vec3d;
    fn add(self, rhs: Self) -> Self::Output {
        Vec3d::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl AddAssign for Vec3d {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl Div for Vec2f {
    type Output = Vec2f;
    fn div(self, rhs: Self) -> Self::Output {
        Vec2f::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl Div<f32> for Vec2f {
    type Output = Vec2f;
    fn div(self, rhs: f32) -> Self::Output {
        Vec2f::new(self.x / rhs, self.y / rhs)
    }
}

impl DivAssign for Vec2f {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl DivAssign<f32> for Vec2f {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Div for Vec3f {
    type Output = Vec3f;
    fn div(self, rhs: Self) -> Self::Output {
        Vec3f::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}

impl Div<f32> for Vec3f {
    type Output = Vec3f;
    fn div(self, rhs: f32) -> Self::Output {
        Vec3f::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl DivAssign for Vec3f {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

impl DivAssign<f32> for Vec3f {
    fn div_assign(&mut self, rhs: f32) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Div for Vec2d {
    type Output = Vec2d;
    fn div(self, rhs: Self) -> Self::Output {
        Vec2d::new(self.x / rhs.x, self.y / rhs.y)
    }
}

impl Div<f64> for Vec2d {
    type Output = Vec2d;
    fn div(self, rhs: f64) -> Self::Output {
        Vec2d::new(self.x / rhs, self.y / rhs)
    }
}

impl DivAssign for Vec2d {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
    }
}

impl DivAssign<f64> for Vec2d {
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
    }
}

impl Div for Vec3d {
    type Output = Vec3d;
    fn div(self, rhs: Self) -> Self::Output {
        Vec3d::new(self.x / rhs.x, self.y / rhs.y, self.z / rhs.z)
    }
}

impl Div<f64> for Vec3d {
    type Output = Vec3d;
    fn div(self, rhs: f64) -> Self::Output {
        Vec3d::new(self.x / rhs, self.y / rhs, self.z / rhs)
    }
}

impl DivAssign for Vec3d {
    fn div_assign(&mut self, rhs: Self) {
        self.x /= rhs.x;
        self.y /= rhs.y;
        self.z /= rhs.z;
    }
}

impl DivAssign<f64> for Vec3d {
    fn div_assign(&mut self, rhs: f64) {
        self.x /= rhs;
        self.y /= rhs;
        self.z /= rhs;
    }
}

impl Mul<f32> for Vec2f {
    type Output = Vec2f;
    fn mul(self, rhs: f32) -> Self::Output {
        Vec2f::new(self.x * rhs, self.y * rhs)
    }
}

impl MulAssign for Vec2f {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl MulAssign<f32> for Vec2f {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Mul for Vec3f {
    type Output = Vec3f;
    fn mul(self, rhs: Self) -> Self::Output {
        Vec3f::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl Mul<f32> for Vec3f {
    type Output = Vec3f;
    fn mul(self, rhs: f32) -> Self::Output {
        Vec3f::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl MulAssign for Vec3f {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl MulAssign<f32> for Vec3f {
    fn mul_assign(&mut self, rhs: f32) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Mul for Vec2d {
    type Output = Vec2d;
    fn mul(self, rhs: Self) -> Self::Output {
        Vec2d::new(self.x * rhs.x, self.y * rhs.y)
    }
}

impl Mul<f64> for Vec2d {
    type Output = Vec2d;
    fn mul(self, rhs: f64) -> Self::Output {
        Vec2d::new(self.x * rhs, self.y * rhs)
    }
}

impl MulAssign for Vec2d {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
    }
}

impl MulAssign<f64> for Vec2d {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
    }
}

impl Mul for Vec3d {
    type Output = Vec3d;
    fn mul(self, rhs: Self) -> Self::Output {
        Vec3d::new(self.x * rhs.x, self.y * rhs.y, self.z * rhs.z)
    }
}

impl Mul<f64> for Vec3d {
    type Output = Vec3d;
    fn mul(self, rhs: f64) -> Self::Output {
        Vec3d::new(self.x * rhs, self.y * rhs, self.z * rhs)
    }
}

impl MulAssign for Vec3d {
    fn mul_assign(&mut self, rhs: Self) {
        self.x *= rhs.x;
        self.y *= rhs.y;
        self.z *= rhs.z;
    }
}

impl MulAssign<f64> for Vec3d {
    fn mul_assign(&mut self, rhs: f64) {
        self.x *= rhs;
        self.y *= rhs;
        self.z *= rhs;
    }
}

impl Sub for Vec2f {
    type Output = Vec2f;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec2f::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Vec2f {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

impl Sub for Vec3f {
    type Output = Vec3f;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec3f::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl SubAssign for Vec3f {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Sub for Vec2d {
    type Output = Vec2d;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec2d::new(self.x - rhs.x, self.y - rhs.y)
    }
}

impl SubAssign for Vec2d {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}
impl Sub for Vec3d {
    type Output = Vec3d;
    fn sub(self, rhs: Self) -> Self::Output {
        Vec3d::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}

impl SubAssign for Vec3d {
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
        self.z -= rhs.z;
    }
}

impl Vec2f {
    pub const ZERO: Vec2f = Vec2f { x: 0.0, y: 0.0 };
    pub const ONE: Vec2f = Vec2f { x: 1.0, y: 1.0 };
    pub const UP: Vec2f = Vec2f { x: 0.0, y: 1.0 };
    pub const DOWN: Vec2f = Vec2f { x: 0.0, y: -1.0 };
    pub const LEFT: Vec2f = Vec2f { x: 1.0, y: 0.0 };
    pub const RIGHT: Vec2f = Vec2f { x: -1.0, y: 0.0 };

    pub fn new(x: f32, y: f32) -> Vec2f { Vec2f { x, y } }

    pub fn from_vec2d(val: Vec2d) -> Vec2f {
        Vec2f {
            x: val.x as f32,
            y: val.y as f32,
        }
    }

    pub fn to_vec2d(&self) -> Vec2d {
        Vec2d {
            x: self.x as f64,
            y: self.y as f64,
        }
    }

    pub fn dot(&mut self, vec: Vec2f) -> f32 { self.x * vec.x + self.y * vec.y }

    pub fn cross(&mut self, vec: Vec2f) -> f32 {
        (self.x * vec.y) - (self.y * vec.x)
    }
}

impl Vec3f {
    pub const ZERO: Vec3f = Vec3f {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const ONE: Vec3f = Vec3f {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    pub const UP: Vec3f = Vec3f {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    pub const DOWN: Vec3f = Vec3f {
        x: 0.0,
        y: -1.0,
        z: 0.0,
    };
    pub const LEFT: Vec3f = Vec3f {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    pub const RIGHT: Vec3f = Vec3f {
        x: -1.0,
        y: 0.0,
        z: 0.0,
    };
    pub const FORWARD: Vec3f = Vec3f {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };
    pub const BACK: Vec3f = Vec3f {
        x: 0.0,
        y: 0.0,
        z: -1.0,
    };

    pub fn new(x: f32, y: f32, z: f32) -> Vec3f { Vec3f { x, y, z } }

    pub fn from_vec3d(val: Vec3d) -> Vec3f {
        Vec3f {
            x: val.x as f32,
            y: val.y as f32,
            z: val.z as f32,
        }
    }

    pub fn to_vec3d(&self) -> Vec3d {
        Vec3d {
            x: self.x as f64,
            y: self.y as f64,
            z: self.z as f64,
        }
    }

    pub fn dot(&mut self, vec: Vec3f) -> f32 {
        self.x * vec.x + self.y * vec.y + self.z * vec.z
    }

    pub fn cross(&mut self, vec: Vec3f) -> Vec3f {
        Vec3f {
            x: (self.y * vec.z) - (self.z * vec.y),
            y: (self.z * vec.x) - (self.x * vec.z),
            z: (self.x * vec.y) - (self.y * vec.x),
        }
    }

    pub fn length_sqr(&mut self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&mut self) -> f32 { self.length_sqr().sqrt() }

    pub fn normalize(&mut self) -> Vec3f {
        let len = self.length();
        Vec3f {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
        }
    }
}

impl Vec2d {
    pub const ZERO: Vec2d = Vec2d { x: 0.0, y: 0.0 };
    pub const ONE: Vec2d = Vec2d { x: 1.0, y: 1.0 };
    pub const UP: Vec2d = Vec2d { x: 0.0, y: 1.0 };
    pub const DOWN: Vec2d = Vec2d { x: 0.0, y: -1.0 };
    pub const LEFT: Vec2d = Vec2d { x: 1.0, y: 0.0 };
    pub const RIGHT: Vec2d = Vec2d { x: -1.0, y: 0.0 };

    pub fn new(x: f64, y: f64) -> Vec2d { Vec2d { x, y } }

    pub fn from_vec2f(val: Vec2f) -> Vec2d {
        Vec2d {
            x: val.x as f64,
            y: val.y as f64,
        }
    }

    pub fn to_vec2f(&self) -> Vec2f {
        Vec2f {
            x: self.x as f32,
            y: self.y as f32,
        }
    }

    pub fn dot(&mut self, vec: Vec2d) -> f64 { self.x * vec.x + self.y * vec.y }

    pub fn cross(&mut self, vec: Vec2d) -> f64 {
        (self.x * vec.y) - (self.y * vec.x)
    }
}

impl Vec3d {
    pub const ZERO: Vec3d = Vec3d {
        x: 0.0,
        y: 0.0,
        z: 0.0,
    };
    pub const ONE: Vec3d = Vec3d {
        x: 1.0,
        y: 1.0,
        z: 1.0,
    };
    pub const UP: Vec3d = Vec3d {
        x: 0.0,
        y: 1.0,
        z: 0.0,
    };
    pub const DOWN: Vec3d = Vec3d {
        x: 0.0,
        y: -1.0,
        z: 0.0,
    };
    pub const LEFT: Vec3d = Vec3d {
        x: 1.0,
        y: 0.0,
        z: 0.0,
    };
    pub const RIGHT: Vec3d = Vec3d {
        x: -1.0,
        y: 0.0,
        z: 0.0,
    };
    pub const FORWARD: Vec3d = Vec3d {
        x: 0.0,
        y: 0.0,
        z: 1.0,
    };
    pub const BACK: Vec3d = Vec3d {
        x: 0.0,
        y: 0.0,
        z: -1.0,
    };

    pub fn new(x: f64, y: f64, z: f64) -> Vec3d { Vec3d { x, y, z } }

    pub fn from_vec3f(val: Vec3f) -> Vec3d {
        Vec3d {
            x: val.x as f64,
            y: val.y as f64,
            z: val.z as f64,
        }
    }

    pub fn to_vec3f(&self) -> Vec3f {
        Vec3f {
            x: self.x as f32,
            y: self.y as f32,
            z: self.z as f32,
        }
    }

    pub fn dot(&mut self, vec: Vec3d) -> f64 {
        self.x * vec.x + self.y * vec.y + self.z * vec.z
    }

    pub fn cross(&mut self, vec: Vec3d) -> Vec3d {
        Vec3d {
            x: (self.y * vec.z) - (self.z * vec.y),
            y: (self.z * vec.x) - (self.x * vec.z),
            z: (self.x * vec.y) - (self.y * vec.x),
        }
    }

    pub fn length_sqr(&mut self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length(&mut self) -> f64 { self.length_sqr().sqrt() }

    pub fn normalize(&mut self) -> Vec3d {
        let len = self.length();
        Vec3d {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
        }
    }
}

impl From<Vec4f> for Vec3f {
    fn from(value: Vec4f) -> Self { Vec3f::new(value.x, value.y, value.z) }
}

impl Vec4f {
    pub const ZERO: Vec4f = Vec4f {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        w: 0.0,
    };
    pub const ONE: Vec4f = Vec4f {
        x: 1.0,
        y: 1.0,
        z: 1.0,
        w: 1.0,
    };

    pub fn new(x: f32, y: f32, z: f32, w: f32) -> Vec4f { Vec4f { x, y, z, w } }

    pub fn length_sqr_xyz(&mut self) -> f32 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn length_xyz(&mut self) -> f32 { self.length_sqr_xyz().sqrt() }

    pub fn normalize_xyz(&mut self) -> Vec4f {
        let len = self.length_xyz();
        Vec4f {
            x: self.x / len,
            y: self.y / len,
            z: self.z / len,
            w: self.w,
        }
    }
}
