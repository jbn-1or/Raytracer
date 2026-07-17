#![allow(dead_code)]

use std::sync::Arc;

use crate::tools::aabb::Aabb;
use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::material::Material;
use crate::tools::ray::Ray;
use crate::tools::rtweekend::{INFINITY, random_double};
use crate::tools::vector3::Vec3;

/// 恒定密度介质（参与介质），模拟烟雾、雾、薄雾等体积效果
/// 内部使用一个边界物体（Hittable）来定义体积的几何形状
pub struct ConstantMedium {
    /// 体积的边界物体
    boundary: Arc<dyn Hittable>,
    /// 负的逆密度（-1/density），预计算以加速指数采样
    neg_inv_density: f64,
    /// 各向同性散射的相位函数材质
    phase_function: Arc<dyn Material>,
}

impl ConstantMedium {
    /// 从边界物体、密度和纹理创建恒定介质（dyn 版本）
    /// # 参数
    /// * `boundary` - 定义体积形状的边界物体
    /// * `density` - 介质密度（正值，越大越不透明）
    /// * `tex` - 控制散射颜色的纹理
    pub fn new(boundary: Arc<dyn Hittable>, density: f64, tex: Arc<dyn Material>) -> Self {
        Self {
            boundary,
            neg_inv_density: -1.0 / density,
            phase_function: tex,
        }
    }

    /// 从边界物体、密度和具体材质类型创建恒定介质（泛型版本）
    /// # 参数
    /// * `boundary` - 定义体积形状的边界物体
    /// * `density` - 介质密度（正值，越大越不透明）
    /// * `tex` - 控制散射颜色的具体材质类型
    pub fn new_static<M: Material + 'static>(
        boundary: Arc<dyn Hittable>,
        density: f64,
        tex: Arc<M>,
    ) -> Self {
        Self {
            boundary,
            neg_inv_density: -1.0 / density,
            phase_function: tex as Arc<dyn Material>,
        }
    }
}

impl Hittable for ConstantMedium {
    /// 检测光线是否与恒定介质相交
    /// 实现思路：找到光线穿过边界的两个交点，在体积内按指数分布采样散射位置
    fn hit(&self, r: &Ray, ray_tmin: f64, ray_tmax: f64, rec: &mut HitRecord) -> bool {
        let mut rec1 = HitRecord::default();
        let mut rec2 = HitRecord::default();

        // 查找光线进入边界的第一个交点（使用全域区间）
        if !self.boundary.hit(r, -INFINITY, INFINITY, &mut rec1) {
            return false;
        }

        // 查找光线离开边界的第二个交点（从 rec1.t 稍后开始）
        if !self.boundary.hit(r, rec1.t + 0.0001, INFINITY, &mut rec2) {
            return false;
        }

        // 将交点的 t 值钳制到 [ray_tmin, ray_tmax] 范围内
        if rec1.t < ray_tmin {
            rec1.t = ray_tmin;
        }
        if rec2.t > ray_tmax {
            rec2.t = ray_tmax;
        }

        // 如果钳制后进入点在离开点之后或相同，则无交错区间
        if rec1.t >= rec2.t {
            return false;
        }

        // 确保 t 值非负
        if rec1.t < 0.0 {
            rec1.t = 0.0;
        }

        // 计算光线在体积内部的实际距离
        let ray_length = r.direction().length();
        let distance_inside_boundary = (rec2.t - rec1.t) * ray_length;

        // 按指数分布随机采样散射距离
        let hit_distance = self.neg_inv_density * random_double().ln();

        // 如果散射距离超出体积边界，光线穿过了体积（未散射）
        if hit_distance > distance_inside_boundary {
            return false;
        }

        // 设置击中点记录
        rec.t = rec1.t + hit_distance / ray_length;
        rec.p = r.at(rec.t);

        // 法线设为任意值（各向同性散射不依赖法线）
        rec.normal = Vec3::new(1.0, 0.0, 0.0);
        rec.front_face = true;
        rec.mat = Some(self.phase_function.clone());

        true
    }

    /// 返回恒定介质的包围盒（即边界物体的包围盒）
    fn bounding_box(&self) -> Aabb {
        self.boundary.bounding_box()
    }
}
