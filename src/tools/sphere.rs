#![allow(dead_code)]

use std::sync::Arc;

use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::material::Material;
use crate::tools::ray::Ray;
use crate::tools::vector3::{Point3, Vec3, dot};

pub struct Sphere {
    /// 球心坐标
    center: Ray,
    /// 球体半径
    radius: f64,
    /// 球体材质
    pub mat: Option<Arc<dyn Material>>,
}

impl Sphere {
    /// 创建一个球体，若半径为负数则自动钳位为 0
    /// # 参数`center`-球心坐标 `radius`-球体半径
    pub fn new(static_center: Point3, radius: f64) -> Self {
        let mut r = radius;
        if radius < 0.0 {
            r = 0.0;
        }
        Self {
            center: Ray::new(static_center, Vec3::new(0.0, 0.0, 0.0)),
            radius: r,
            mat: None,
        }
    }

    /// 创建一个带有材质的球体，若半径为负数则自动钳位为 0
    /// # 参数`center`-球心坐标 `radius`-球体半径 `mat`-球体材质
    pub fn new_with_material(static_center: Point3, radius: f64, mat: Arc<dyn Material>) -> Self {
        let mut r = radius;
        if radius < 0.0 {
            r = 0.0;
        }
        Self {
            center: Ray::new(static_center, Vec3::new(0.0, 0.0, 0.0)),
            radius: r,
            mat: Some(mat),
        }
    }

    pub fn new_move_with_material(
        center1: Point3,
        center2: Point3,
        radius: f64,
        mat: Arc<dyn Material>,
    ) -> Self {
        let mut r = radius;
        if radius < 0.0 {
            r = 0.0;
        }
        Self {
            center: Ray::new(center1, center2 - center1),
            radius: r,
            mat: Some(mat),
        }
    }
}

impl Hittable for Sphere {
    /// 检测光线是否与球体相交，若相交则更新 HitRecord
    /// # 参数`r`-入射光线 `ray_tmin（max）`-光线参数 t 的最小（大）阈值 `rec`-储存HitRecord
    fn hit(&self, r: &Ray, ray_tmin: f64, ray_tmax: f64, rec: &mut HitRecord) -> bool {
        let current_center: Point3 = self.center.at(r.time());
        let oc = current_center - r.origin();
        let a = r.direction().length_squared();
        let half_b = dot(r.direction(), oc);
        let c = oc.length_squared() - self.radius * self.radius;

        let discriminant = half_b * half_b - a * c;

        if discriminant < 0.0 {
            return false;
        }

        let sqrtd = discriminant.sqrt();

        // 寻找范围内最近的根
        let mut root = (half_b - sqrtd) / a;
        if root <= ray_tmin || root >= ray_tmax {
            root = (half_b + sqrtd) / a;
            if root <= ray_tmin || root >= ray_tmax {
                return false;
            }
        }

        rec.t = root;
        rec.p = r.at(rec.t);
        let outward_normal = (rec.p - current_center) / self.radius;
        rec.set_face_normal(r, outward_normal);
        rec.mat = self.mat.clone();

        true
    }
}
