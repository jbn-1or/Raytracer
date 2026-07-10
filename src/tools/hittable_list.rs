#![allow(dead_code)]

use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::ray::Ray;

pub struct HittableList {
    /// 可击中物体的列表
    objects: Vec<Box<dyn Hittable>>,
}

impl HittableList {
    /// 创建一个空的物体列表
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    /// 清空物体列表
    pub fn clear(&mut self) {
        self.objects.clear();
    }

    /// 向列表中添加一个物体
    /// # 参数`object`-待添加的物体（盒装 trait 对象）
    pub fn add(&mut self, object: Box<dyn Hittable>) {
        self.objects.push(object);
    }
}

impl Hittable for HittableList {
    /// 遍历所有物体，检测光线是否与其中任意一个相交，并返回最近的交点
    /// # 参数`r`-入射光线 `ray_tmin（max）`-光线参数 t 的最小（大）阈值 `rec`-储存HitRecord
    fn hit(&self, r: &Ray, ray_tmin: f64, ray_tmax: f64, rec: &mut HitRecord) -> bool {
        let mut temp_rec: HitRecord = HitRecord::default();
        let mut hit_anything = false;
        let mut closest_so_far = ray_tmax;

        for object in &self.objects {
            if object.hit(r, ray_tmin, closest_so_far, &mut temp_rec) {
                hit_anything = true;
                closest_so_far = temp_rec.t;
                *rec = temp_rec.clone();
            }
        }

        hit_anything
    }
}
