#![allow(dead_code)]

use crate::tools::interval::Interval;

use super::vector3::Vec3;
use image::Rgb;

pub type Color = Vec3;

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
}
