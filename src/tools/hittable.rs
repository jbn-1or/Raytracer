#![allow(dead_code)]
#![allow(unused_variables)]

use crate::tools::vector3::{Point3, Vec3, dot};

use super::ray::Ray;

#[derive(Clone, Copy)]
pub struct HitRecord {
    pub p: Point3,
    pub normal: Vec3,
    pub t: f64,
    front_face: bool,
}

impl Default for HitRecord {
    fn default() -> Self {
        Self {
            p: Point3::zero(),
            normal: Vec3::zero(),
            t: 0.0,
            front_face: false,
        }
    }
}

impl HitRecord {
    // Sets the hit record normal vector.
    // NOTE: the parameter `outward_normal` is assumed to have unit length
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3) {
        self.front_face = dot(r.direction(), outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }
}
pub trait Hittable {
    fn hit(&self, r: &Ray, ray_tmin: f64, ray_tmax: f64, rec: &mut HitRecord) -> bool {
        false
    }
}
