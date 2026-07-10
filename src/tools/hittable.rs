#![allow(dead_code)]
#![allow(unused_variables)]

use std::sync::Arc;

use crate::tools::vector3::{Point3, Vec3, dot};

use super::material::Material;
use super::ray::Ray;

/// 光线与物体的交点记录，存储交点位置、法线、材质等相交信息
#[derive(Clone)]
pub struct HitRecord {
    /// 交点位置
    pub p: Point3,
    /// 交点法线向量（单位向量）
    pub normal: Vec3,
    /// 光线参数 t，即交点距光线起点的距离
    pub t: f64,
    /// 物体的材质
    pub mat: Option<Arc<dyn Material>>,
    /// 光线是否从表面外部射入
    front_face: bool,
}

impl Default for HitRecord {
    /// 默认构造函数，创建一个空的 HitRecord
    fn default() -> Self {
        Self {
            p: Point3::zero(),
            normal: Vec3::zero(),
            t: 0.0,
            mat: None,
            front_face: false,
        }
    }
}

impl HitRecord {
    /// 根据光线方向与法线的点积，设置前/背面标志和法线方向
    /// # 参数`r`-入射光线 `outward_normal`-表面朝外的法线（需为单位向量）
    pub fn set_face_normal(&mut self, r: &Ray, outward_normal: Vec3) {
        self.front_face = dot(r.direction(), outward_normal) < 0.0;
        self.normal = if self.front_face {
            outward_normal
        } else {
            -outward_normal
        };
    }
}

/// 可被光线击中的物体抽象接口
pub trait Hittable {
    /// 检测光线是否与物体相交
    /// # 参数`r`-入射光线 `ray_tmin（max）`-光线参数 t 的最小（大）阈值 `rec`-储存HitRecord
    fn hit(&self, r: &Ray, ray_tmin: f64, ray_tmax: f64, rec: &mut HitRecord) -> bool {
        false
    }
}
