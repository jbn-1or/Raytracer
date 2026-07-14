#![allow(dead_code)]

use crate::tools::rtweekend::random_int;
use crate::tools::vector3::{Point3, Vec3, dot, unit_vector};

const POINT_COUNT: usize = 256;

/// 基于值的噪声（Perlin 噪声的简化版本）
/// 使用预计算的随机向量数组和三个排列数组，
/// 通过将空间坐标离散化并查找排列后的随机向量来生成噪声。
/// 在格点上使用随机单位向量，并通过点积计算插值，
/// 使得噪声的最小值和最大值不再恰好位于整数点上。
pub struct Perlin {
    randvec: [Vec3; POINT_COUNT],
    perm_x: [usize; POINT_COUNT],
    perm_y: [usize; POINT_COUNT],
    perm_z: [usize; POINT_COUNT],
}

impl Perlin {
    /// 创建一个新的 Perlin 噪声实例
    /// 初始化随机单位向量数组和三个坐标轴方向的排列数组
    pub fn new() -> Self {
        let mut randvec = [Vec3::zero(); POINT_COUNT];
        for item in randvec.iter_mut() {
            *item = unit_vector(Vec3::random_range(-1.0, 1.0));
        }

        Self {
            randvec,
            perm_x: Self::generate_perm(),
            perm_y: Self::generate_perm(),
            perm_z: Self::generate_perm(),
        }
    }

    /// 计算点 `p` 处的噪声值
    /// 使用随机向量和点积进行插值，使噪声更平滑自然
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

        let mut c = [[[Vec3::zero(); 2]; 2]; 2];

        for (di, c_di) in c.iter_mut().enumerate() {
            for (dj, c_dj) in c_di.iter_mut().enumerate() {
                for (dk, c_val) in c_dj.iter_mut().enumerate() {
                    *c_val = self.randvec[self.perm_x[((i + di as i32) & 255) as usize]
                        ^ self.perm_y[((j + dj as i32) & 255) as usize]
                        ^ self.perm_z[((k + dk as i32) & 255) as usize]];
                }
            }
        }

        Self::perlin_interp(&c, u, v, w)
    }

    /// Perlin 插值函数
    /// 对 2×2×2 网格的八个顶点进行平滑插值，
    /// 使用随机向量的点积使噪声最小/最大值偏离格点
    fn perlin_interp(c: &[[[Vec3; 2]; 2]; 2], u: f64, v: f64, w: f64) -> f64 {
        let uu = u * u * (3.0 - 2.0 * u);
        let vv = v * v * (3.0 - 2.0 * v);
        let ww = w * w * (3.0 - 2.0 * w);
        let mut accum = 0.0;

        for (i, c_i) in c.iter().enumerate() {
            let i_f = i as f64;
            for (j, c_j) in c_i.iter().enumerate() {
                let j_f = j as f64;
                for (k, c_vec) in c_j.iter().enumerate() {
                    let k_f = k as f64;
                    let weight_v = Vec3::new(u - i_f, v - j_f, w - k_f);
                    accum += (i_f * uu + (1.0 - i_f) * (1.0 - uu))
                        * (j_f * vv + (1.0 - j_f) * (1.0 - vv))
                        * (k_f * ww + (1.0 - k_f) * (1.0 - ww))
                        * dot(*c_vec, weight_v);
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
