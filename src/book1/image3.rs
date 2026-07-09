#![allow(dead_code)]

use crate::tools::color::Color;
use crate::tools::ray::Ray;
use crate::tools::render_utils::{create_progress_bar, prepare_output_path, save_image};
use crate::tools::vector3::{Point3, Vec3, dot, unit_vector};
use image::{ImageBuffer, RgbImage};

fn hit_sphere(center: Point3, radius: f64, r: &Ray) -> bool {
    let oc = center - r.origin();
    let a = dot(r.direction(), r.direction());
    let b = -2.0 * dot(r.direction(), oc);
    let c = dot(oc, oc) - radius * radius;
    let discriminant = b * b - 4.0 * a * c;
    discriminant >= 0.0
}

fn ray_color(r: &Ray) -> Vec3 {
    if hit_sphere(Point3::new(0.0, 0.0, -1.0), 0.5, r) {
        return Color::new(1.0, 0.0, 0.0);
    }

    let unit_direction = unit_vector(r.direction());
    let a = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
}

pub fn render() {
    let path = prepare_output_path("output/book1/image3.png");

    // 设定照片视口尺寸
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;

    let mut image_height: u32 = ((image_width) as f64 / aspect_ratio) as u32;
    if image_height < 1 {
        image_height = 1;
    }
    let focal_length = 1.0;
    let viewport_height = 2.0;
    let viewport_width: f64 = viewport_height * (image_width as f64 / image_height as f64);
    let camera_center = Point3::new(0.0, 0.0, 0.0);

    // 计算视口水平方向与垂直向下方向边缘的向量。
    let viewport_u: Vec3 = Vec3::new(viewport_width, 0.0, 0.0);
    let viewport_v: Vec3 = Vec3::new(0.0, -viewport_height, 0.0);

    // 计算像素与像素之间的水平和垂直差值向量。
    let pixel_delta_u = viewport_u / (image_width) as f64;
    let pixel_delta_v = viewport_v / (image_height) as f64;

    // 计算左上角像素的位置。
    let viewport_upper_left =
        camera_center - Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
    let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    // 产生新图片
    let mut img: RgbImage = ImageBuffer::new(image_width, image_height);

    let progress = create_progress_bar((image_height * image_width) as u64);

    // 渲染图片
    for j in 0..image_height {
        for i in 0..image_width {
            let pixel_center =
                pixel00_loc + (i as f64 * pixel_delta_u) + (j as f64 * pixel_delta_v);
            let ray_direction = pixel_center - camera_center;
            let r = Ray::new(camera_center, ray_direction);

            let pixel_color = ray_color(&r);
            let pixel = img.get_pixel_mut(i, j);
            Color::write_color(pixel_color, pixel);
            progress.inc(1);
        }
    }

    progress.finish();

    save_image(&img, &path);
}
