use serde::{Deserialize, Serialize};
use std::ops::Add;
use std::ops::Mul;

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
#[repr(C)]
pub struct Color<T> {
    pub r: T,
    pub g: T,
    pub b: T,
    pub a: T,
}

#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct ColorBytes {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color<u8> {
    pub fn get_bytes(self) -> Result<ColorBytes, i32> {
        Ok(ColorBytes {
            r: self.r,
            g: self.g,
            b: self.b,
            a: self.a,
        })
    }
}

impl Add for Color<u8> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r.saturating_add(rhs.r),
            g: self.g.saturating_add(rhs.g),
            b: self.b.saturating_add(rhs.b),
            a: self.a.saturating_add(rhs.a),
        }
    }
}

impl Add for Color<f32> {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r + rhs.r,
            g: self.g + rhs.g,
            b: self.b + rhs.b,
            a: self.a + rhs.a,
        }
    }
}

impl Mul for Color<u8> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r.saturating_mul(rhs.r),
            g: self.g.saturating_mul(rhs.g),
            b: self.b.saturating_mul(rhs.b),
            a: self.a.saturating_mul(rhs.a),
        }
    }
}

impl Mul for Color<f32> {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            r: self.r * rhs.r,
            g: self.g * rhs.g,
            b: self.b * rhs.b,
            a: self.a * rhs.a,
        }
    }
}

impl Mul<f32> for Color<f32> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            r: self.r * rhs,
            g: self.g * rhs,
            b: self.b * rhs,
            a: self.a * rhs,
        }
    }
}

impl Mul<Color<f32>> for Color<u8> {
    type Output = Self;

    fn mul(self, rhs: Color<f32>) -> Self::Output {
        Self {
            r: (self.r as f32 * rhs.r).clamp(0.0, 255.0) as u8,
            g: (self.g as f32 * rhs.g).clamp(0.0, 255.0) as u8,
            b: (self.b as f32 * rhs.b).clamp(0.0, 255.0) as u8,
            a: (self.a as f32 * rhs.a).clamp(0.0, 255.0) as u8,
        }
    }
}

impl Mul<f32> for Color<u8> {
    type Output = Self;

    fn mul(self, rhs: f32) -> Self::Output {
        Self {
            r: (self.r as f32 * rhs).clamp(0.0, 255.0) as u8,
            g: (self.g as f32 * rhs).clamp(0.0, 255.0) as u8,
            b: (self.b as f32 * rhs).clamp(0.0, 255.0) as u8,
            a: (self.a as f32 * 1.0).clamp(0.0, 255.0) as u8,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub enum Material {
    Matte {
        color: Color<u8>,
    },
    Specular {
        color: Color<u8>,
        specular: f32,
        reflectiveness: f32,
    },
}
