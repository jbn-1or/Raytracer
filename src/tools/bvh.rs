#![allow(dead_code)]

use std::rc::Rc;

use crate::tools::aabb::Aabb;
use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::hittable_list::HittableList;
use crate::tools::interval::Interval;
use crate::tools::ray::Ray;
/// 盒子比较函数类型
type BoxCompareFn = dyn Fn(&Rc<dyn Hittable>, &Rc<dyn Hittable>) -> std::cmp::Ordering;

/// BVH（包围体层次结构 / Bounding Volume Hierarchy）节点，实现加速光线求交的层次包围盒树
/// 与 `hittable_list` 一样是一个容器，接收光线求交查询时通过层次包围盒快速筛选物体。
pub struct BvhNode {
    /// 左子节点
    left: Rc<dyn Hittable>,
    /// 右子节点（当物体数量为 1 时与左子节点指向相同物体）
    right: Rc<dyn Hittable>,
    /// 当前节点的包围盒（包含左右子节点的合并包围盒）
    bbox: Aabb,
}

impl BvhNode {
    /// 从 `HittableList` 构建 BVH 树
    /// 此构造函数创建物体的隐式拷贝，然后递归构建层次包围盒。
    /// 拷贝的生命周期仅在此构造函数内有效，仅需保留最终的层次结构。
    pub fn from_list(list: HittableList) -> Self {
        let objects: Vec<Rc<dyn Hittable>> = list.objects.into_iter().map(Rc::from).collect();
        Self::from_objects(&objects, 0, objects.len())
    }

    /// 从物体切片递归构建 BVH 树
    fn from_objects(objects: &[Rc<dyn Hittable>], start: usize, end: usize) -> Self {
        // 构建对象跨度的包围盒
        let mut bbox = Aabb::EMPTY;
        for obj in objects[start..end].iter() {
            bbox = Aabb::new_with_boxes(&bbox, &obj.bounding_box());
        }

        let axis = bbox.longest_axis();

        let comparator: &BoxCompareFn = if axis == 0 {
            &Self::box_x_compare
        } else if axis == 1 {
            &Self::box_y_compare
        } else {
            &Self::box_z_compare
        };

        let object_span = end - start;

        let (left, right) = if object_span == 1 {
            // 只有一个物体，左右都指向同一个物体
            let obj = objects[start].clone();
            (obj.clone(), obj)
        } else if object_span == 2 {
            // 两个物体，各分一个
            let left = objects[start].clone();
            let right = objects[start + 1].clone();
            (left, right)
        } else {
            // 三个及以上物体，排序后递归构建
            let mut sorted_objects: Vec<Rc<dyn Hittable>> = objects[start..end].to_vec();
            sorted_objects.sort_by(comparator);

            let mid = object_span / 2;
            let left_node = Self::from_objects(&sorted_objects, 0, mid);
            let right_node = Self::from_objects(&sorted_objects, mid, object_span);
            (
                Rc::new(left_node) as Rc<dyn Hittable>,
                Rc::new(right_node) as Rc<dyn Hittable>,
            )
        };

        Self { left, right, bbox }
    }

    /// 通用盒子比较函数，比较两个物体在指定轴上的包围盒最小值
    fn box_compare(
        a: &Rc<dyn Hittable>,
        b: &Rc<dyn Hittable>,
        axis_index: usize,
    ) -> std::cmp::Ordering {
        let a_axis_interval = a.bounding_box().axis_interval(axis_index);
        let b_axis_interval = b.bounding_box().axis_interval(axis_index);
        a_axis_interval
            .min
            .partial_cmp(&b_axis_interval.min)
            .unwrap()
    }

    /// X 轴比较函数
    fn box_x_compare(a: &Rc<dyn Hittable>, b: &Rc<dyn Hittable>) -> std::cmp::Ordering {
        Self::box_compare(a, b, 0)
    }

    /// Y 轴比较函数
    fn box_y_compare(a: &Rc<dyn Hittable>, b: &Rc<dyn Hittable>) -> std::cmp::Ordering {
        Self::box_compare(a, b, 1)
    }

    /// Z 轴比较函数
    fn box_z_compare(a: &Rc<dyn Hittable>, b: &Rc<dyn Hittable>) -> std::cmp::Ordering {
        Self::box_compare(a, b, 2)
    }
}

impl Hittable for BvhNode {
    /// 检测光线是否与 BVH 节点中的物体相交，返回最近的节点
    /// 首先检查当前节点的包围盒是否被击中，若未击中则直接返回。
    /// 若击中，则递归检测左右子节点，并利用已找到的最近交点优化右子树的搜索范围。
    fn hit(&self, r: &Ray, ray_tmin: f64, ray_tmax: f64, rec: &mut HitRecord) -> bool {
        if !self.bbox.hit(r, Interval::new(ray_tmin, ray_tmax)) {
            return false;
        }

        let hit_left = self.left.hit(r, ray_tmin, ray_tmax, rec);
        let right_tmax = if hit_left { rec.t } else { ray_tmax };
        let hit_right = self.right.hit(r, ray_tmin, right_tmax, rec);

        hit_left || hit_right
    }

    /// 返回当前节点的包围盒
    fn bounding_box(&self) -> Aabb {
        self.bbox
    }
}
