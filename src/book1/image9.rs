#![allow(dead_code)]

use crate::tools::camera::Camera;
use crate::tools::color::Color;
use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::hittable_list::HittableList;
use crate::tools::ray::Ray;
use crate::tools::render_utils::{create_progress_bar, prepare_output_path, save_image};
use crate::tools::rtweekend::INFINITY;
use crate::tools::sphere::Sphere;
use crate::tools::vector3::{Point3, Vec3, random_on_hemisphere, unit_vector};
use image::{ImageBuffer, RgbImage};

fn ray_color(r: &Ray, depth: u32, world: &dyn Hittable) -> Vec3 {
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    let mut rec: HitRecord = HitRecord::default();

    if world.hit(r, 0.001, INFINITY, &mut rec) {
        let direction = random_on_hemisphere(rec.normal);
        return 0.5 * ray_color(&Ray::new(rec.p, direction), depth - 1, world);
    }

    let unit_direction = unit_vector(r.direction());
    let a = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
}

pub fn render() {
    let path = prepare_output_path("output/book1/image9.png");

    // World
    let mut world: HittableList = HittableList::new();
    world.add(Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)));
    world.add(Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)));

    // Camera
    let mut cam: Camera = Camera::new(16.0 / 9.0, 400, 100);
    cam.max_depth = 50;
    let image_width = cam.image_width;
    let image_height = cam.image_height();
    let pixel_samples_scale = cam.pixel_samples_scale();

    // 产生新图片
    let mut img: RgbImage = ImageBuffer::new(image_width, image_height);

    let progress = create_progress_bar((image_height * image_width) as u64);

    // 渲染图片
    for j in 0..image_height {
        for i in 0..image_width {
            let mut pixel_color: Color = Color::zero();
            for _sample in 0..cam.samples_per_pixel {
                let r = cam.get_ray(i, j);
                pixel_color += ray_color(&r, cam.max_depth, &world);
            }
            let pixel = img.get_pixel_mut(i, j);
            Color::write_color(pixel_color * pixel_samples_scale, pixel);
            progress.inc(1);
        }
    }

    progress.finish();

    save_image(&img, &path);
}
