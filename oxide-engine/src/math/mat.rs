//! Matrix types.

use std::ops::Mul;

use crate::math::vec::Vec3f;

/// 4x4 matrix of floats.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
#[repr(C)]
pub struct Matrix4f(pub [[f32; 4]; 4]);

impl Mul for Matrix4f {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut output = Matrix4f::indentity();
        for i in (0..4).step_by(1) {
            for j in (0..4).step_by(1) {
                output.0[i][j] = 0.0;
                for k in (0..4).step_by(1) {
                    output.0[i][j] += self.0[k][j] * rhs.0[i][k];
                }
            }
        }
        output
    }
}

impl Matrix4f {
    pub fn new(val: [[f32; 4]; 4]) -> Matrix4f {
        Matrix4f([
            [val[0][0], val[1][0], val[2][0], val[3][0]],
            [val[0][1], val[1][1], val[2][1], val[3][1]],
            [val[0][2], val[1][2], val[2][2], val[3][2]],
            [val[0][3], val[1][3], val[2][3], val[3][3]],
        ])
    }

    /// An identity matrix.
    pub fn indentity() -> Matrix4f {
        Matrix4f::new([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// A translation matrix.
    pub fn translation(vec: Vec3f) -> Matrix4f {
        Matrix4f::new([
            [1.0, 0.0, 0.0, vec.x],
            [0.0, 1.0, 0.0, vec.y],
            [0.0, 0.0, 1.0, vec.z],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// A scale matrix.
    pub fn scale(vec: Vec3f) -> Matrix4f {
        Matrix4f::new([
            [vec.x, 0.0, 0.0, 0.0],
            [0.0, vec.y, 0.0, 0.0],
            [0.0, 0.0, vec.z, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Rotation around x-axis matrix.
    pub fn rotation_x(angle: f32) -> Matrix4f {
        Matrix4f::new([
            [1.0, 0.0, 0.0, 0.0],
            [0.0, angle.cos(), -angle.sin(), 0.0],
            [0.0, angle.sin(), angle.cos(), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Rotation around y-axis matrix.
    pub fn rotation_y(angle: f32) -> Matrix4f {
        Matrix4f::new([
            [angle.cos(), 0.0, -angle.sin(), 0.0],
            [0.0, 1.0, 0.0, 0.0],
            [angle.sin(), 0.0, angle.cos(), 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Rotation around z-axis matrix.
    pub fn rotation_z(angle: f32) -> Matrix4f {
        Matrix4f::new([
            [angle.cos(), -angle.sin(), 0.0, 0.0],
            [angle.sin(), angle.cos(), 0.0, 0.0],
            [0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }

    /// Perspective matrix.
    pub fn perspective(fovy: f32, aspect: f32, near: f32, far: f32) -> Matrix4f {
        let f = 1.0 / (fovy / 2.0).tan();
        let a = far / (far - near);
        Matrix4f::new([
            [f / aspect, 0.0, 0.0, 0.0],
            [0.0, -f, 0.0, 0.0],
            [0.0, 0.0, a, -near * a],
            [0.0, 0.0, 1.0, 0.0],
        ])
    }

    /// Right-handed look-at matrix.
    pub fn look_at(mut eye: Vec3f, mut dir: Vec3f, mut up: Vec3f) -> Matrix4f {
        let mut f = dir.normalize();
        let mut u = f.cross(up.normalize()).normalize();
        let v = u.cross(f);

        Matrix4f::new([
            [u.x, u.y, u.z, -eye.dot(u)],
            [v.x, v.y, v.z, -eye.dot(v)],
            [f.x, f.y, f.z, -eye.dot(f)],
            [0.0, 0.0, 0.0, 1.0],
        ])
    }
}
