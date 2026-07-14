#![allow(dead_code)]

use std::sync::Arc;

use crate::tools::{color::Color, vector3::Point3};

pub trait Texture: Send + Sync {
    fn value(&self, _u: f64, _v: f64, _p: Point3) -> Color {
        Color::default()
    }
}

pub struct SolidColor {
    albedo: Color,
}

impl SolidColor {
    pub fn new(red: f64, green: f64, blue: f64) -> Self {
        Self {
            albedo: (Color::new(red, green, blue)),
        }
    }

    pub fn from_color(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: Point3) -> Color {
        self.albedo
    }
}

pub struct CheckerTexture {
    inv_scale: f64,
    even: Arc<dyn Texture>,
    odd: Arc<dyn Texture>,
}

impl CheckerTexture {
    pub fn new(scale: f64, even: Arc<dyn Texture>, odd: Arc<dyn Texture>) -> Self {
        Self {
            inv_scale: 1.0 / scale,
            even,
            odd,
        }
    }

    pub fn from_another(scale: f64, c1: Color, c2: Color) -> Self {
        Self {
            inv_scale: 1.0 / scale,
            even: Arc::new(SolidColor::from_color(c1)),
            odd: Arc::new(SolidColor::from_color(c2)),
        }
    }
}

impl Texture for CheckerTexture {
    fn value(&self, _u: f64, _v: f64, p: Point3) -> Color {
        let x_integer = (self.inv_scale * p.x()).floor() as i32;
        let y_integer = (self.inv_scale * p.y()).floor() as i32;
        let z_integer = (self.inv_scale * p.z()).floor() as i32;

        let is_even = (x_integer + y_integer + z_integer) % 2 == 0;

        if is_even {
            self.even.value(_u, _v, p)
        } else {
            self.odd.value(_u, _v, p)
        }
    }
}
