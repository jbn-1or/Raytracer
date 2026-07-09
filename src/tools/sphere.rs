#![allow(dead_code)]

use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::ray::Ray;
use crate::tools::vector3::{Point3, dot};

pub struct Sphere {
    center: Point3, // 球心坐标
    radius: f64,    // 球体半径
}

impl Sphere {
    /// 创建一个球体，若半径为负数则自动钳位为 0
    /// # 参数`center`-球心坐标 `radius`-球体半径
    pub fn new(center: Point3, radius: f64) -> Self {
        let mut r = radius;
        if radius < 0.0 {
            r = 0.0;
        }
        Self { center, radius: r }
    }
}

impl Hittable for Sphere {
    /// 检测光线是否与球体相交，若相交则更新 HitRecord
    /// # 参数`r`-入射光线 `ray_tmin（max）`-光线参数 t 的最小（大）阈值 `rec`-储存HitRecord
    fn hit(&self, r: &Ray, ray_tmin: f64, ray_tmax: f64, rec: &mut HitRecord) -> bool {
        let oc = self.center - r.origin();
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
        let outward_normal = (rec.p - self.center) / self.radius;
        rec.set_face_normal(r, outward_normal);

        true
    }
}
