#![allow(dead_code)]

use super::rtweekend::INFINITY;

/// 区间类，表示 [min, max] 范围内的数值
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
