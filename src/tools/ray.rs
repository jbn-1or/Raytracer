#![allow(dead_code)]

use super::vector3::{Point3, Vec3};

pub struct Ray {
    /// 光线起点
    orig: Point3,
    /// 光线方向向量
    dir: Vec3,
}

impl Ray {
    /// 从起点和方向创建一条光线
    /// # 参数`origin`-光线起点 `direction`-光线方向向量
    pub const fn new(origin: Point3, direction: Vec3) -> Self {
        Self {
            orig: origin,
            dir: direction,
        }
    }

    /// 创建一条位于原点、方向为零向量的光线（用于空初始化）
    pub const fn zero() -> Self {
        Self {
            orig: Point3::zero(),
            dir: Vec3::zero(),
        }
    }

    /// 获取光线起点
    pub const fn origin(&self) -> Point3 {
        self.orig
    }

    /// 获取光线方向
    pub const fn direction(&self) -> Vec3 {
        self.dir
    }

    /// 计算光线在参数 t 处的坐标位置
    /// # 参数`t`-光线参数（沿方向向量的距离）
    pub fn at(&self, t: f64) -> Point3 {
        self.orig + t * self.dir
    }
}
