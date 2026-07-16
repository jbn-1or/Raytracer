#![allow(dead_code)]
#![allow(unused_variables)]

use std::rc::Rc;
use std::sync::Arc;

use crate::tools::aabb::Aabb;
use crate::tools::rtweekend::degrees_to_radians;
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
    /// 光线-物体击中点的 u,v 表面坐标
    pub u: f64,
    pub v: f64,
    /// 光线是否从表面外部射入
    pub front_face: bool,
}

impl Default for HitRecord {
    /// 默认构造函数，创建一个空的 HitRecord
    fn default() -> Self {
        Self {
            p: Point3::zero(),
            normal: Vec3::zero(),
            t: 0.0,
            mat: None,
            u: 0.0,
            v: 0.0,
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

/// 平移变换：将子物体沿 offset 向量平移
pub struct Translate {
    object: Rc<dyn Hittable>,
    offset: Vec3,
    bbox: Aabb,
}

impl Translate {
    /// 创建一个平移变换对象
    /// # 参数`object`-待平移的子物体 `offset`-平移向量
    pub fn new(object: Rc<dyn Hittable>, offset: Vec3) -> Self {
        let bbox = object.bounding_box() + offset;
        Self {
            object,
            offset,
            bbox,
        }
    }
}

impl Hittable for Translate {
    /// 检测光线是否与平移后的物体相交
    /// 实现：将光线向后平移 offset，检测子物体，再将交点向前平移 offset
    fn hit(&self, r: &Ray, ray_tmin: f64, ray_tmax: f64, rec: &mut HitRecord) -> bool {
        // 将光线向后平移 offset
        let offset_r = Ray::new_with_time(r.origin() - self.offset, r.direction(), r.time());

        if !self.object.hit(&offset_r, ray_tmin, ray_tmax, rec) {
            return false;
        }

        // 将交点向前平移 offset
        rec.p += self.offset;

        true
    }

    /// 返回平移后的包围盒
    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}

/// 绕 Y 轴旋转：将子物体绕 Y 轴旋转指定角度（角度制）
pub struct RotateY {
    object: Rc<dyn Hittable>,
    sin_theta: f64,
    cos_theta: f64,
    bbox: Aabb,
}

impl RotateY {
    /// 创建一个绕 Y 轴旋转的变换对象（角度制）
    pub fn new(object: Rc<dyn Hittable>, angle: f64) -> Self {
        let radians = degrees_to_radians(angle);
        let sin_theta = radians.sin();
        let cos_theta = radians.cos();

        let bbox = object.bounding_box();

        let mut min = Point3::new(f64::INFINITY, f64::INFINITY, f64::INFINITY);
        let mut max = Point3::new(f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY);

        for i in 0..2 {
            for j in 0..2 {
                for k in 0..2 {
                    let x = i as f64 * bbox.x.max + (1 - i) as f64 * bbox.x.min;
                    let y = j as f64 * bbox.y.max + (1 - j) as f64 * bbox.y.min;
                    let z = k as f64 * bbox.z.max + (1 - k) as f64 * bbox.z.min;

                    let newx = cos_theta * x + sin_theta * z;
                    let newz = -sin_theta * x + cos_theta * z;

                    let tester = Vec3::new(newx, y, newz);

                    min = Point3::new(
                        f64::min(min.x(), tester.x()),
                        f64::min(min.y(), tester.y()),
                        f64::min(min.z(), tester.z()),
                    );
                    max = Point3::new(
                        f64::max(max.x(), tester.x()),
                        f64::max(max.y(), tester.y()),
                        f64::max(max.z(), tester.z()),
                    );
                }
            }
        }

        let bbox = Aabb::new_with_points(min, max);

        Self {
            object,
            sin_theta,
            cos_theta,
            bbox,
        }
    }
}

impl Hittable for RotateY {
    fn hit(&self, r: &Ray, ray_tmin: f64, ray_tmax: f64, rec: &mut HitRecord) -> bool {
        // 将光线从世界空间变换到对象空间（绕 Y 轴旋转 -θ）
        let origin = Point3::new(
            self.cos_theta * r.origin().x() - self.sin_theta * r.origin().z(),
            r.origin().y(),
            self.sin_theta * r.origin().x() + self.cos_theta * r.origin().z(),
        );

        let direction = Vec3::new(
            self.cos_theta * r.direction().x() - self.sin_theta * r.direction().z(),
            r.direction().y(),
            self.sin_theta * r.direction().x() + self.cos_theta * r.direction().z(),
        );

        let rotated_r = Ray::new_with_time(origin, direction, r.time());

        // 检测在对象空间中是否存在相交
        if !self.object.hit(&rotated_r, ray_tmin, ray_tmax, rec) {
            return false;
        }

        // 将交点从对象空间变换回世界空间（绕 Y 轴旋转 +θ）
        rec.p = Point3::new(
            self.cos_theta * rec.p.x() + self.sin_theta * rec.p.z(),
            rec.p.y(),
            -self.sin_theta * rec.p.x() + self.cos_theta * rec.p.z(),
        );

        rec.normal = Vec3::new(
            self.cos_theta * rec.normal.x() + self.sin_theta * rec.normal.z(),
            rec.normal.y(),
            -self.sin_theta * rec.normal.x() + self.cos_theta * rec.normal.z(),
        );

        true
    }

    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}

/// 可被光线击中的物体抽象接口
pub trait Hittable {
    /// 检测光线是否与物体相交
    /// # 参数`r`-入射光线 `ray_tmin（max）`-光线参数 t 的最小（大）阈值 `rec`-储存HitRecord
    fn hit(&self, r: &Ray, ray_tmin: f64, ray_tmax: f64, rec: &mut HitRecord) -> bool {
        false
    }

    /// 返回物体的轴对齐包围盒（AABB），默认返回空包围盒
    fn bounding_box(&self) -> Aabb {
        Aabb::default()
    }
}
