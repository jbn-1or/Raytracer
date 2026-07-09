#![allow(dead_code)]

use crate::tools::render_utils::{create_progress_bar, prepare_output_path, save_image};
use image::{ImageBuffer, RgbImage};

pub fn render() {
    let path = prepare_output_path("output/book1/image1.png");

    let width = 256;
    let height = 256;

    let mut img: RgbImage = ImageBuffer::new(width, height);

    let progress = create_progress_bar((height * width) as u64);

    for j in (0..height).rev() {
        for i in 0..width {
            let pixel = img.get_pixel_mut(i, j);
            let r: f64 = (i as f64) / ((width - 1) as f64) * 255.999;
            let g: f64 = (j as f64) / ((height - 1) as f64) * 255.999;
            let b: f64 = 0.25 * 255.999;
            *pixel = image::Rgb([r as u8, g as u8, b as u8]);
        }
        progress.inc(1);
    }

    progress.finish();

    save_image(&img, &path);
}
