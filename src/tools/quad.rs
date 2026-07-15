#![allow(dead_code)]

use std::sync::Arc;

use crate::tools::aabb::Aabb;
use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::material::Material;
use crate::tools::ray::Ray;
use crate::tools::vector3::{Point3, Vec3, cross, dot, unit_vector};

/// 四边形（平行四边形）图元，由起始角点 Q 和两条边向量 u、v 定义
#[allow(non_snake_case)]
pub struct Quad {
    /// 起始角点
    Q: Point3,
    /// 第一条边向量
    u: Vec3,
    /// 第二条边向量
    v: Vec3,
    /// 缓存向量 w = n / dot(n, n)，用于快速计算平面坐标 (α, β)
    w: Vec3,
    /// 材质
    mat: Option<Arc<dyn Material>>,
    /// 包围盒
    bbox: Aabb,
    /// 平面单位法向量
    normal: Vec3,
    /// 平面方程常数 D（满足 n·p = D）
    d: f64,
}

impl Quad {
    /// 创建一个新的四边形
    /// # 参数
    /// * `Q` - 起始角点
    /// * `u` - 第一条边向量
    /// * `v` - 第二条边向量
    /// * `mat` - 材质
    #[allow(non_snake_case)]
    pub fn new(Q: Point3, u: Vec3, v: Vec3, mat: Arc<dyn Material>) -> Self {
        let n = cross(u, v);
        let normal = unit_vector(n);
        let d = dot(normal, Q);
        let w = n / dot(n, n);

        let mut quad = Self {
            Q,
            u,
            v,
            w,
            mat: Some(mat),
            bbox: Aabb::default(),
            normal,
            d,
        };
        quad.set_bounding_box();
        quad
    }

    /// 计算并设置四边形的包围盒（四个顶点的 AABB 的合并）
    fn set_bounding_box(&mut self) {
        let bbox_diagonal1 = Aabb::new_with_points(self.Q, self.Q + self.u + self.v);
        let bbox_diagonal2 = Aabb::new_with_points(self.Q + self.u, self.Q + self.v);
        self.bbox = Aabb::new_with_boxes(&bbox_diagonal1, &bbox_diagonal2);
    }

    /// 判断平面坐标 (a, b) 是否位于四边形内部，若是则设置 `rec` 的 UV 坐标
    /// # 参数
    /// * `a` - 沿 u 方向的平面坐标（对应 α）
    /// * `b` - 沿 v 方向的平面坐标（对应 β）
    /// * `rec` - 光线相交记录（仅击中时更新 u、v 值）
    /// # 返回
    /// * `true` - 点在四边形内部；`false` - 点在四边形外部
    fn is_interior(a: f64, b: f64, rec: &mut HitRecord) -> bool {
        // 四边形内部要求坐标在 [0, 1] 范围内
        if !(0.0..=1.0).contains(&a) || !(0.0..=1.0).contains(&b) {
            return false;
        }

        rec.u = a;
        rec.v = b;
        true
    }
}

impl Hittable for Quad {
    /// 检测光线是否与四边形相交
    fn hit(&self, r: &Ray, ray_tmin: f64, ray_tmax: f64, rec: &mut HitRecord) -> bool {
        let denom = dot(self.normal, r.direction());

        // 若光线与平面平行，则无交点
        if denom.abs() < 1e-8 {
            return false;
        }

        // 计算光线与平面的交点参数 t
        let t = (self.d - dot(self.normal, r.origin())) / denom;

        // 检查 t 是否在有效范围内
        if t <= ray_tmin || t >= ray_tmax {
            return false;
        }

        // 计算交点
        let intersection = r.at(t);

        // 将交点转换到平面坐标系，计算平面坐标 (α, β)
        let planar_hitpt_vector = intersection - self.Q;
        let alpha = dot(self.w, cross(planar_hitpt_vector, self.v));
        let beta = dot(self.w, cross(self.u, planar_hitpt_vector));

        // 判断交点是否在四边形内部
        if !Self::is_interior(alpha, beta, rec) {
            return false;
        }

        // 填充相交记录
        rec.t = t;
        rec.p = intersection;
        rec.mat = self.mat.clone();
        rec.set_face_normal(r, self.normal);

        true
    }

    /// 返回四边形的包围盒
    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}
