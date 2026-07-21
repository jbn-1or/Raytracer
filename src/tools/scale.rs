#![allow(dead_code)]

use std::sync::Arc;

use crate::tools::aabb::Aabb;
use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::ray::Ray;
use crate::tools::vector3::{Point3, unit_vector};

/// 缩放变换：将子物体按给定因子缩放（各向同性）
pub struct Scale<H: Hittable> {
    object: Arc<H>,
    scale: f64,
    bbox: Aabb,
}

impl<H: Hittable> Scale<H> {
    /// 创建一个缩放变换对象，scale 必须 > 0
    pub fn new(object: Arc<H>, scale: f64) -> Self {
        assert!(scale > 0.0, "Scale factor must be positive");
        let inner = object.bounding_box();
        let bbox = Aabb::new_with_points(
            Point3::new(
                inner.x.min * scale,
                inner.y.min * scale,
                inner.z.min * scale,
            ),
            Point3::new(
                inner.x.max * scale,
                inner.y.max * scale,
                inner.z.max * scale,
            ),
        );
        Self {
            object,
            scale,
            bbox,
        }
    }
}

impl<H: Hittable> Hittable for Scale<H> {
    /// 检测光线与缩放后的物体是否相交
    /// 思路：将光线除以缩放因子（变换到物体空间），求交后再将交点乘以缩放因子
    fn hit(&self, r: &Ray, ray_tmin: f64, ray_tmax: f64, rec: &mut HitRecord) -> bool {
        let inv_scale = 1.0 / self.scale;

        // 将光线变换到物体空间
        let scaled_origin = r.origin() * inv_scale;
        let scaled_direction = r.direction(); // 方向不变，长度变化由 t 自动补偿
        let scaled_r = Ray::new_with_time(scaled_origin, scaled_direction, r.time());

        if !self.object.hit(&scaled_r, ray_tmin, ray_tmax, rec) {
            return false;
        }

        // 将交点变换回世界空间
        rec.p *= self.scale;

        // 各向同性缩放下法线方向不变，只需重新归一化
        rec.normal = unit_vector(rec.normal);

        true
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}
