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
    /// 使用三线性插值平滑结果
    pub fn noise(&self, p: &Point3) -> f64 {
        let mut u = p.x() - p.x().floor();
        let mut v = p.y() - p.y().floor();
        let mut w = p.z() - p.z().floor();

        u = u * u * (3.0 - 2.0 * u);
        v = v * v * (3.0 - 2.0 * v);
        w = w * w * (3.0 - 2.0 * w);

        let i = p.x().floor() as i32;
        let j = p.y().floor() as i32;
        let k = p.z().floor() as i32;

        let mut c = [[[0.0_f64; 2]; 2]; 2];

        for (di, c_di) in c.iter_mut().enumerate() {
            for (dj, c_dj) in c_di.iter_mut().enumerate() {
                for (dk, c_val) in c_dj.iter_mut().enumerate() {
                    *c_val = self.randfloat[self.perm_x[((i + di as i32) & 255) as usize]
                        ^ self.perm_y[((j + dj as i32) & 255) as usize]
                        ^ self.perm_z[((k + dk as i32) & 255) as usize]];
                }
            }
        }

        Self::trilinear_interp(&c, u, v, w)
    }

    /// 三线性插值函数
    /// 对 2×2×2 网格的八个顶点进行平滑插值
    fn trilinear_interp(c: &[[[f64; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let mut accum = 0.0;
        for (i, c_i) in c.iter().enumerate() {
            for (j, c_j) in c_i.iter().enumerate() {
                for (k, val) in c_j.iter().enumerate() {
                    let i_f = i as f64;
                    let j_f = j as f64;
                    let k_f = k as f64;
                    accum += (i_f * u + (1.0 - i_f) * (1.0 - u))
                        * (j_f * v + (1.0 - j_f) * (1.0 - v))
                        * (k_f * w + (1.0 - k_f) * (1.0 - w))
                        * val;
                }
            }
        }
        accum
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
