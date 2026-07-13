#![allow(dead_code)]

use crate::tools::{interval::Interval, ray::Ray, vector3::Point3};

/// 轴对齐包围盒（Axis-Aligned Bounding Box），由 x/y/z 三个区间的笛卡尔积定义
#[derive(Clone, Copy)]
pub struct Aabb {
    /// x 轴区间
    pub x: Interval,
    /// y 轴区间
    pub y: Interval,
    /// z 轴区间
    pub z: Interval,
}

impl Default for Aabb {
    fn default() -> Self {
        Self {
            x: Interval::EMPTY,
            y: Interval::EMPTY,
            z: Interval::EMPTY,
        }
    }
}

impl Aabb {
    /// 从三个轴区间直接构造 AABB
    pub fn new(x: Interval, y: Interval, z: Interval) -> Self {
        Self { x, y, z }
    }

    /// 从两个 AABB 构造一个能紧密包围二者的新 AABB
    pub fn new_with_boxes(box0: &Self, box1: &Self) -> Self {
        Self {
            x: Interval::from_intervals(box0.x, box1.x),
            y: Interval::from_intervals(box0.y, box1.y),
            z: Interval::from_intervals(box0.z, box1.z),
        }
    }

    /// 以 a、b 为对角顶点构造 AABB，自动处理大小顺序
    pub fn new_with_points(a: Point3, b: Point3) -> Self {
        let intervalx: Interval = if a[0] > b[0] {
            Interval::new(b[0], a[0])
        } else {
            Interval::new(a[0], b[0])
        };
        let intervaly: Interval = if a[1] > b[1] {
            Interval::new(b[1], a[1])
        } else {
            Interval::new(a[1], b[1])
        };
        let intervalz: Interval = if a[2] > b[2] {
            Interval::new(b[2], a[2])
        } else {
            Interval::new(a[2], b[2])
        };

        Self {
            x: intervalx,
            y: intervaly,
            z: intervalz,
        }
    }

    /// 返回指定轴的区间：0→x, 1→y, 2→z
    pub fn axis_interval(&self, n: usize) -> Interval {
        if n == 1 {
            self.y
        } else if n == 2 {
            self.z
        } else {
            self.x
        }
    }

    /// 返回最长轴的索引：0→x, 1→y, 2→z
    pub fn longest_axis(&self) -> usize {
        if self.x.size() > self.y.size() {
            if self.x.size() > self.z.size() { 0 } else { 2 }
        } else if self.y.size() > self.z.size() {
            1
        } else {
            2
        }
    }

    /// 使用平板法（slab method）检测光线是否与 AABB 相交
    pub fn hit(&self, r: &Ray, mut ray_t: Interval) -> bool {
        let ray_orig = r.origin();
        let ray_dir = r.direction();

        for axis in 0..3 {
            let ax = self.axis_interval(axis);
            let adinv = 1.0 / ray_dir[axis];

            let t0 = (ax.min - ray_orig[axis]) * adinv;
            let t1 = (ax.max - ray_orig[axis]) * adinv;

            if t0 < t1 {
                if t0 > ray_t.min {
                    ray_t.min = t0;
                }
                if t1 < ray_t.max {
                    ray_t.max = t1;
                }
            } else {
                if t1 > ray_t.min {
                    ray_t.min = t1;
                }
                if t0 < ray_t.max {
                    ray_t.max = t0;
                }
            }

            if ray_t.max <= ray_t.min {
                return false;
            }
        }
        true
    }

    /// 空包围盒常量（所有轴区间为空）
    pub const EMPTY: Self = Self {
        x: Interval::EMPTY,
        y: Interval::EMPTY,
        z: Interval::EMPTY,
    };

    /// 全空间包围盒常量（所有轴区间为全空间）
    pub const UNIVERSE: Self = Self {
        x: Interval::UNIVERSE,
        y: Interval::UNIVERSE,
        z: Interval::UNIVERSE,
    };
}
