#![allow(dead_code)]

use crate::tools::interval::Interval;

use super::vector3::Vec3;
use image::Rgb;

pub type Color = Vec3;

/// 将线性颜色分量转换为伽马空间（应用 sqrt 校正）
fn linear_to_gamma(linear_component: f64) -> f64 {
    if linear_component > 0.0 {
        return linear_component.sqrt();
    }
    0.0
}

impl Color {
    /// 将像素颜色值写入 RGB 像素缓冲区
    /// # 参数`pixel_color`-像素颜色向量（各分量在[0,1]范围） `pixel`-目标RGB像素的可变引用
    pub fn write_color(pixel_color: Color, pixel: &mut Rgb<u8>) {
        let x = pixel_color.x();
        let y = pixel_color.y();
        let z = pixel_color.z();

        let intensity: Interval = Interval::new(0.000, 0.999);
        let r = (256.0 * intensity.clamp(x)) as u32;
        let g = (256.0 * intensity.clamp(y)) as u32;
        let b = (256.0 * intensity.clamp(z)) as u32;
        *pixel = image::Rgb([r as u8, g as u8, b as u8]);
    }

    /// 对颜色应用伽马校正后写入 RGB 像素缓冲区
    /// # 参数`pixel_color`-像素颜色向量（各分量在[0,1]范围） `pixel`-目标RGB像素的可变引用
    pub fn write_color_gamma(pixel_color: Color, pixel: &mut Rgb<u8>) {
        let mut x = pixel_color.x();
        let mut y = pixel_color.y();
        let mut z = pixel_color.z();

        x = linear_to_gamma(x);
        y = linear_to_gamma(y);
        z = linear_to_gamma(z);

        let intensity: Interval = Interval::new(0.000, 0.999);
        let r = (256.0 * intensity.clamp(x)) as u32;
        let g = (256.0 * intensity.clamp(y)) as u32;
        let b = (256.0 * intensity.clamp(z)) as u32;
        *pixel = image::Rgb([r as u8, g as u8, b as u8]);
    }
}
