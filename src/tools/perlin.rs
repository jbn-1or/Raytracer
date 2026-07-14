#![allow(dead_code)]

use crate::tools::rtweekend::{random_double, random_int};
use crate::tools::vector3::Point3;

const POINT_COUNT: usize = 256;

/// 基于值的噪声（Perlin 噪声的简化版本）
/// 使用预计算的随机浮点数数组和三个排列数组，
/// 通过将空间坐标离散化并查找排列后的随机值来生成噪声。
pub struct Perlin {
    randfloat: [f64; POINT_COUNT],
    perm_x: [usize; POINT_COUNT],
    perm_y: [usize; POINT_COUNT],
    perm_z: [usize; POINT_COUNT],
}

impl Perlin {
    /// 创建一个新的 Perlin 噪声实例
    /// 初始化随机浮点数组和三个坐标轴方向的排列数组
    pub fn new() -> Self {
        let mut randfloat = [0.0_f64; POINT_COUNT];
        for item in randfloat.iter_mut() {
            *item = random_double();
        }

        Self {
            randfloat,
            perm_x: Self::generate_perm(),
            perm_y: Self::generate_perm(),
            perm_z: Self::generate_perm(),
        }
    }

    /// 计算点 `p` 处的噪声值
    /// 通过将坐标映射到离散网格，使用排列数组索引随机浮点数组
    pub fn noise(&self, p: &Point3) -> f64 {
        let i = ((4.0 * p.x()) as i32 & 255) as usize;
        let j = ((4.0 * p.y()) as i32 & 255) as usize;
        let k = ((4.0 * p.z()) as i32 & 255) as usize;

        self.randfloat[self.perm_x[i] ^ self.perm_y[j] ^ self.perm_z[k]]
    }

    /// 生成一个 [0, POINT_COUNT) 的排列数组
    fn generate_perm() -> [usize; POINT_COUNT] {
        let mut p = [0_usize; POINT_COUNT];
        for (i, item) in p.iter_mut().enumerate() {
            *item = i;
        }
        Self::permute(&mut p);
        p
    }

    /// 对长度为 `n` 的数组进行 Fisher-Yates 洗牌
    fn permute(p: &mut [usize]) {
        let n = p.len();
        for i in (1..n).rev() {
            let target = random_int(0, i as u32) as usize;
            p.swap(i, target);
        }
    }
}
