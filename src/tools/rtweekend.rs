#![allow(unused_variables)]
#![allow(dead_code)]

// 常量
/// 无穷大，用于表示光线未与物体相交时的最大距离
pub const INFINITY: f64 = f64::INFINITY;
use std::f64::consts::PI;

// 工具函数
/// 将角度转换为弧度
/// # 参数`degrees`-角度值
#[inline]
pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}

/// 返回 [0,1) 范围内的随机实数
#[inline]
pub fn random_double() -> f64 {
    rand::random::<f64>()
}

/// 返回 [min,max) 范围内的随机实数
#[inline]
pub fn random_double_range(min: f64, max: f64) -> f64 {
    min + (max - min) * random_double()
}
