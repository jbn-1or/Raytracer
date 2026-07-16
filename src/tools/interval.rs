#![allow(dead_code)]

use super::rtweekend::INFINITY;

/// 区间类，表示 [min, max] 范围内的数值
#[derive(Clone, Copy)]
pub struct Interval {
    /// 区间下界
    pub min: f64,
    /// 区间上界
    pub max: f64,
}

impl Interval {
    /// 使用给定的最小值和最大值创建区间
    pub fn new(min: f64, max: f64) -> Self {
        Self { min, max }
    }

    /// 返回区间的长度（max - min）
    pub fn size(&self) -> f64 {
        self.max - self.min
    }

    /// 判断 x 是否在区间内（包含端点）
    pub fn contains(&self, x: f64) -> bool {
        self.min <= x && x <= self.max
    }

    /// 判断 x 是否在区间内（不包含端点）
    pub fn surrounds(&self, x: f64) -> bool {
        self.min < x && x < self.max
    }

    /// 选取区间中最接近x的数（区间钳制函数）
    pub fn clamp(&self, x: f64) -> f64 {
        if x < self.min {
            return self.min;
        };
        if x > self.max {
            return self.max;
        };
        x
    }

    /// 将区间向两端各扩展 delta/2，返回新区间。用于 BVH 节点的容差扩展
    pub fn expand(&self, delta: f64) -> Self {
        let padding = delta / 2.0;
        Interval::new(self.min - padding, self.max + padding)
    }

    /// 从两个区间创建能紧密包围二者的新区间
    pub fn from_intervals(a: Self, b: Self) -> Self {
        Self {
            min: if a.min <= b.min { a.min } else { b.min },
            max: if a.max >= b.max { a.max } else { b.max },
        }
    }

    /// 空区间常量
    pub const EMPTY: Interval = Interval {
        min: INFINITY,
        max: -INFINITY,
    };

    /// 全空间区间常量
    pub const UNIVERSE: Interval = Interval {
        min: -INFINITY,
        max: INFINITY,
    };
}

impl Default for Interval {
    /// 默认构造函数，创建一个空区间（min > max）
    fn default() -> Self {
        Self {
            min: INFINITY,
            max: -INFINITY,
        }
    }
}

impl std::ops::Add<f64> for Interval {
    type Output = Interval;

    /// 区间整体加上一个位移，返回新区间
    fn add(self, displacement: f64) -> Interval {
        Interval::new(self.min + displacement, self.max + displacement)
    }
}

impl std::ops::Add<Interval> for f64 {
    type Output = Interval;

    /// f64 + Interval，交换律，委托给 Interval + f64
    fn add(self, ival: Interval) -> Interval {
        ival + self
    }
}
