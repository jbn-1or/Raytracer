#![allow(unused_variables)]
#![allow(dead_code)]

// 常量
pub const INFINITY: f64 = f64::INFINITY;
use std::f64::consts::PI;

// 工具函数
#[inline]
pub fn degrees_to_radians(degrees: f64) -> f64 {
    degrees * PI / 180.0
}
