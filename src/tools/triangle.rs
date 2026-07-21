#![allow(dead_code)]

use std::sync::Arc;

use crate::tools::aabb::Aabb;
use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::material::Material;
use crate::tools::ray::Ray;
use crate::tools::vector3::{Point3, Vec3, cross, dot, unit_vector};

/// 三角形几何体，使用 Möller–Trumbore 算法进行光线求交
pub struct Triangle {
    /// 3 个顶点
    vertices: [Point3; 3],
    /// 可选的三顶点法线（用于平滑着色），若为 None 则使用面法线
    normals: Option<[Vec3; 3]>,
    /// 材质
    pub mat: Option<Arc<dyn Material>>,
    /// 包围盒
    bbox: Aabb,
}

impl Triangle {
    /// 从 3 个顶点创建三角形（使用面法线）
    pub fn new(v0: Point3, v1: Point3, v2: Point3) -> Self {
        let min = Point3::new(
            v0.x().min(v1.x()).min(v2.x()),
            v0.y().min(v1.y()).min(v2.y()),
            v0.z().min(v1.z()).min(v2.z()),
        );
        let max = Point3::new(
            v0.x().max(v1.x()).max(v2.x()),
            v0.y().max(v1.y()).max(v2.y()),
            v0.z().max(v1.z()).max(v2.z()),
        );
        Self {
            vertices: [v0, v1, v2],
            normals: None,
            mat: None,
            bbox: Aabb::new_with_points(min, max),
        }
    }

    /// 从 3 个顶点和逐顶点法线创建三角形（平滑着色）
    pub fn new_with_normals(
        v0: Point3,
        v1: Point3,
        v2: Point3,
        n0: Vec3,
        n1: Vec3,
        n2: Vec3,
    ) -> Self {
        let min = Point3::new(
            v0.x().min(v1.x()).min(v2.x()),
            v0.y().min(v1.y()).min(v2.y()),
            v0.z().min(v1.z()).min(v2.z()),
        );
        let max = Point3::new(
            v0.x().max(v1.x()).max(v2.x()),
            v0.y().max(v1.y()).max(v2.y()),
            v0.z().max(v1.z()).max(v2.z()),
        );
        Self {
            vertices: [v0, v1, v2],
            normals: Some([n0, n1, n2]),
            mat: None,
            bbox: Aabb::new_with_points(min, max),
        }
    }

    /// 设置材质（构建器模式）
    pub fn with_material(mut self, mat: Arc<dyn Material>) -> Self {
        self.mat = Some(mat);
        self
    }

    /// 计算面法线（不属于任何顶点的几何法线）
    pub fn face_normal(&self) -> Vec3 {
        let edge1 = self.vertices[1] - self.vertices[0];
        let edge2 = self.vertices[2] - self.vertices[0];
        unit_vector(cross(edge1, edge2))
    }
}

impl Hittable for Triangle {
    /// Möller–Trumbore 光线-三角形求交算法
    /// 参考：Fast, Minimum Storage Ray/Triangle Intersection (Tomas Möller & Ben Trumbore, 1997)
    fn hit(&self, r: &Ray, ray_tmin: f64, ray_tmax: f64, rec: &mut HitRecord) -> bool {
        let v0 = self.vertices[0];
        let v1 = self.vertices[1];
        let v2 = self.vertices[2];

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;

        let pvec = cross(r.direction(), edge2);
        let det = dot(edge1, pvec);

        // 若行列式接近零，光线与三角形平面平行
        if det.abs() < 1e-8 {
            return false;
        }

        let inv_det = 1.0 / det;
        let tvec = r.origin() - v0;
        let u = dot(tvec, pvec) * inv_det;

        // 重心坐标 u 不在 [0, 1] 内则无交点
        if !(0.0..=1.0).contains(&u) {
            return false;
        }

        let qvec = cross(tvec, edge1);
        let v = dot(r.direction(), qvec) * inv_det;

        // 重心坐标 v 不在 [0, 1] 内，或 u+v > 1 则无交点
        if v < 0.0 || u + v > 1.0 {
            return false;
        }

        let t = dot(edge2, qvec) * inv_det;

        // t 在有效范围内
        if t <= ray_tmin || t >= ray_tmax {
            return false;
        }

        // 计算交点
        rec.t = t;
        rec.p = r.at(t);

        // 计算法线：如果有逐顶点法线则使用重心坐标插值（平滑着色），否则使用面法线
        let outward_normal: Vec3 = match &self.normals {
            Some(ns) => {
                let w = 1.0 - u - v;
                unit_vector(ns[0] * w + ns[1] * u + ns[2] * v)
            }
            None => self.face_normal(),
        };

        rec.set_face_normal(r, outward_normal);

        // UV 坐标：直接用重心坐标 (u, v) 传给材质系统
        rec.u = u;
        rec.v = v;
        rec.mat = self.mat.clone();

        true
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}
