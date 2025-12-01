//! Quaternion types (not yet implemented)

use std::ops::{Mul, MulAssign};

use crate::math::{mat::Matrix4f, vec::Vec3f};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quat {
    i: f32,
    j: f32,
    k: f32,
    r: f32,
}

impl Mul for Quat {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        let magnitude = self.r * self.r
            + self.i * self.i
            + self.j * self.j
            + self.k * self.k;
        Quat::new(
            (self.r * rhs.r - self.i * rhs.i - self.j * rhs.j - self.k * rhs.k)
                / magnitude,
            (self.r * rhs.i + self.i * rhs.r - self.j * rhs.k + self.k * rhs.j)
                / magnitude,
            (self.r * rhs.j + self.j * rhs.r + self.i * rhs.k - self.k * rhs.i)
                / magnitude,
            (self.r * rhs.k + self.k * rhs.r - self.i * rhs.j + self.j * rhs.i)
                / magnitude,
        )
    }
}

impl MulAssign for Quat {
    fn mul_assign(&mut self, rhs: Self) { *self = *self * rhs; }
}

impl Mul<Vec3f> for Quat {
    type Output = Vec3f;
    fn mul(self, rhs: Vec3f) -> Self::Output {
        let vec_quat = Quat::new(0.0, rhs.x, rhs.y, rhs.z);
        let quat_mul = self * vec_quat * self.invert();
        Vec3f::new(quat_mul.i, quat_mul.j, quat_mul.k)
    }
}

impl Mul<Quat> for Vec3f {
    type Output = Vec3f;
    fn mul(self, rhs: Quat) -> Self::Output {
        let vec_quat = Quat::new(0.0, self.x, self.y, self.z);
        let quat_mul = rhs.invert() * vec_quat * rhs;
        Vec3f::new(quat_mul.i, quat_mul.j, quat_mul.k)
    }
}

impl MulAssign<Quat> for Vec3f {
    fn mul_assign(&mut self, rhs: Quat) { *self = *self * rhs; }
}

impl Quat {
    pub const IDENTITY: Quat = Quat {
        r: 1.0,
        i: 0.0,
        j: 0.0,
        k: 0.0,
    };

    pub fn new(r: f32, i: f32, j: f32, k: f32) -> Quat { Quat { i, j, k, r } }

    pub fn invert(self) -> Quat { Quat::new(self.r, -self.i, -self.j, -self.k) }

    pub fn into_matrix(&self) -> Matrix4f {
        Matrix4f::new([
            [
                1.0 - 2.0 * self.j.powi(2) - 2.0 * self.k.powi(2),
                2.0 * self.i * self.j - 2.0 * self.r * self.k,
                2.0 * self.i * self.k + 2.0 * self.r * self.j,
                0.0,
            ],
            [
                2.0 * self.i * self.j + 2.0 * self.r * self.k,
                1.0 - 2.0 * self.i.powi(2) - 2.0 * self.k.powi(2),
                2.0 * self.j * self.k - 2.0 * self.r * self.i,
                0.0,
            ],
            [
                2.0 * self.i * self.k - 2.0 * self.r * self.j,
                2.0 * self.j * self.k + 2.0 * self.r * self.i,
                1.0 - 2.0 * self.i.powi(2) - 2.0 * self.j.powi(2),
                0.0,
            ],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    pub fn from_euler(angles: Vec3f) -> Quat {
        let cu = (angles.x / 2.0).cos();
        let cv = (angles.y / 2.0).cos();
        let cw = (angles.z / 2.0).cos();
        let su = (angles.x / 2.0).sin();
        let sv = (angles.y / 2.0).sin();
        let sw = (angles.z / 2.0).sin();

        Quat::new(
            cu * cv * cw + su * sv * sw,
            su * cv * cw - cu * sv * sw,
            cu * sv * cw + su * cv * sw,
            cu * cv * sw - su * sv * cw,
        )
    }
}
