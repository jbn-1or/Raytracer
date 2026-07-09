#![allow(dead_code)]

use std::fmt;
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, MulAssign, Neg, Sub};

/// 三维向量，底层存储为 [f64; 3]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vec3 {
    e: [f64; 3], // 三个分量的数组，依次为 x、y、z
}

/// 三维空间中的点，基于 Vec3 实现
pub type Point3 = Vec3;

impl Vec3 {
    // 构造函数
    /// 创建一个新的三维向量
    /// # 参数`e0`-x分量 `e1`-y分量 `e2`-z分量
    pub const fn new(e0: f64, e1: f64, e2: f64) -> Self {
        Self { e: [e0, e1, e2] }
    }

    /// 创建一个所有分量为零的向量
    pub const fn zero() -> Self {
        Self { e: [0.0, 0.0, 0.0] }
    }

    // 访问器
    /// 获取 x 分量
    pub fn x(&self) -> f64 {
        self.e[0]
    }
    /// 获取 y 分量
    pub fn y(&self) -> f64 {
        self.e[1]
    }
    /// 获取 z 分量
    pub fn z(&self) -> f64 {
        self.e[2]
    }

    // 几何方法
    /// 计算向量的模长
    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    /// 计算向量模长的平方（避免了开方运算）
    pub fn length_squared(&self) -> f64 {
        self.e[0] * self.e[0] + self.e[1] * self.e[1] + self.e[2] * self.e[2]
    }
}

impl Default for Vec3 {
    fn default() -> Self {
        Self::zero()
    }
}

// 运算符: -v
impl Neg for Vec3 {
    type Output = Vec3;

    fn neg(self) -> Self::Output {
        Vec3::new(-self.e[0], -self.e[1], -self.e[2])
    }
}

// 运算符: v[i] (不可变)
impl Index<usize> for Vec3 {
    type Output = f64;

    fn index(&self, i: usize) -> &Self::Output {
        &self.e[i]
    }
}

// 运算符: v[i] (可变)
impl IndexMut<usize> for Vec3 {
    fn index_mut(&mut self, i: usize) -> &mut Self::Output {
        &mut self.e[i]
    }
}

// -运算符: v += u
impl AddAssign for Vec3 {
    fn add_assign(&mut self, rhs: Self) {
        self.e[0] += rhs.e[0];
        self.e[1] += rhs.e[1];
        self.e[2] += rhs.e[2];
    }
}

// 运算符: v *= t
impl MulAssign<f64> for Vec3 {
    fn mul_assign(&mut self, t: f64) {
        self.e[0] *= t;
        self.e[1] *= t;
        self.e[2] *= t;
    }
}

// 运算符: v /= t
impl DivAssign<f64> for Vec3 {
    fn div_assign(&mut self, t: f64) {
        *self *= 1.0 / t;
    }
}

// 运算符: u + v
impl Add for Vec3 {
    type Output = Vec3;

    fn add(self, rhs: Self) -> Self::Output {
        Vec3::new(
            self.e[0] + rhs.e[0],
            self.e[1] + rhs.e[1],
            self.e[2] + rhs.e[2],
        )
    }
}

// 运算符: u - v
impl Sub for Vec3 {
    type Output = Vec3;

    fn sub(self, rhs: Self) -> Self::Output {
        Vec3::new(
            self.e[0] - rhs.e[0],
            self.e[1] - rhs.e[1],
            self.e[2] - rhs.e[2],
        )
    }
}

// 运算符: u * v (逐分量乘)
impl Mul for Vec3 {
    type Output = Vec3;

    fn mul(self, rhs: Self) -> Self::Output {
        Vec3::new(
            self.e[0] * rhs.e[0],
            self.e[1] * rhs.e[1],
            self.e[2] * rhs.e[2],
        )
    }
}

// 运算符: t * v
impl Mul<Vec3> for f64 {
    type Output = Vec3;

    fn mul(self, v: Vec3) -> Self::Output {
        Vec3::new(self * v.e[0], self * v.e[1], self * v.e[2])
    }
}

// 运算符: v * t
impl Mul<f64> for Vec3 {
    type Output = Vec3;

    fn mul(self, t: f64) -> Self::Output {
        t * self
    }
}

// 运算符: v / t
impl Div<f64> for Vec3 {
    type Output = Vec3;

    fn div(self, t: f64) -> Self::Output {
        (1.0 / t) * self
    }
}

// Display: 打印为 "e0 e1 e2"
impl fmt::Display for Vec3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.e[0], self.e[1], self.e[2])
    }
}

/// 计算两个向量的点积
/// # 参数`u`-第一个向量 `v`-第二个向量
pub fn dot(u: Vec3, v: Vec3) -> f64 {
    u.e[0] * v.e[0] + u.e[1] * v.e[1] + u.e[2] * v.e[2]
}

/// 计算两个向量的叉积，结果是一个垂直于 u、v 的向量
/// # 参数`u`-第一个向量 `v`-第二个向量
pub fn cross(u: Vec3, v: Vec3) -> Vec3 {
    Vec3::new(
        u.e[1] * v.e[2] - u.e[2] * v.e[1],
        u.e[2] * v.e[0] - u.e[0] * v.e[2],
        u.e[0] * v.e[1] - u.e[1] * v.e[0],
    )
}

/// 获取向量的单位向量
/// # 参数`v`-输入向量
pub fn unit_vector(v: Vec3) -> Vec3 {
    v / v.length()
}
