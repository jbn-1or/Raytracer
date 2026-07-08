use super::vector3::Vec3;
use image::Rgb;

pub type Color = Vec3;

impl Color {
    pub fn write_color(pixel_color: &Color, pixel: &mut Rgb<u8>) {
        let x = pixel_color.x();
        let y = pixel_color.y();
        let z = pixel_color.z();
        let r = (255.999 * x) as u32;
        let g = (255.999 * y) as u32;
        let b = (255.999 * z) as u32;
        *pixel = image::Rgb([r as u8, g as u8, b as u8]);
    }
}
