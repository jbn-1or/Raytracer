#![allow(dead_code)]

use std::sync::Arc;

use crate::tools::camera::Camera;
use crate::tools::color::Color;
use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::hittable_list::HittableList;
use crate::tools::material::{Lambertian, Material, Metal};
use crate::tools::ray::Ray;
use crate::tools::render_utils::{create_progress_bar, prepare_output_path, render_parallel_gamma, save_image};
use crate::tools::rtweekend::INFINITY;
use crate::tools::sphere::Sphere;
use crate::tools::vector3::{Point3, Vec3, unit_vector};
use image::{ImageBuffer, RgbImage};

fn ray_color(r: &Ray, depth: u32, world: &dyn Hittable) -> Vec3 {
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    let mut rec: HitRecord = HitRecord::default();

    if world.hit(r, 0.001, INFINITY, &mut rec) {
        let mut scattered: Ray = Ray::zero();
        let mut attenuation: Color = Color::zero();

        if let Some(ref mat) = rec.mat {
            if mat.scatter(r, &rec, &mut attenuation, &mut scattered) {
                return attenuation * ray_color(&scattered, depth - 1, world);
            }
        }
        return Color::new(0.0, 0.0, 0.0);
    }

    let unit_direction = unit_vector(r.direction());
    let a = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
}

pub fn render() {
    let path = prepare_output_path("output/book1/image13.png");

    // World
    let mut world: HittableList = HittableList::new();

    // 地面：灰色朗伯材质
    let ground_material: Arc<dyn Material> = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.0)));
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(0.0, -100.5, -1.0),
        100.0,
        ground_material,
    )));

    // 中间：灰色朗伯材质
    let center_material = Arc::new(Lambertian::new(Color::new(0.1, 0.2, 0.5)));
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(0.0, 0.0, -1.2),
        0.5,
        center_material,
    )));
    // 左侧：白色金属材质
    let left_material: Arc<dyn Material> = Arc::new(Metal::new(Color::new(0.8, 0.8, 0.8)));
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(-1.0, 0.0, -1.0),
        0.5,
        left_material,
    )));

    // 右侧：金色金属材质
    let right_material: Arc<dyn Material> = Arc::new(Metal::new(Color::new(0.8, 0.6, 0.2)));
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(1.0, 0.0, -1.0),
        0.5,
        right_material,
    )));

    // Camera
    let mut cam: Camera = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 400;
    cam.samples_per_pixel = 100;
    cam.max_depth = 50;
    cam.initialize();
    let image_width = cam.image_width;
    let image_height = cam.image_height();
    let pixel_samples_scale = cam.pixel_samples_scale();

    // 产生新图片
    let mut img: RgbImage = ImageBuffer::new(image_width, image_height);

    let progress = create_progress_bar((image_height * image_width) as u64);

    let samples = cam.samples_per_pixel;
    let max_depth = cam.max_depth;
    // 渲染图片
    render_parallel_gamma(&mut img, image_width, image_height, move |i, j| {
        let mut pixel_color: Color = Color::zero();
        for _sample in 0..samples {
            let r = cam.get_ray(i, j);
            pixel_color += ray_color(&r, max_depth, &world);
        }
        pixel_color * pixel_samples_scale
    }, &progress);

    progress.finish();

    save_image(&img, &path);
}
