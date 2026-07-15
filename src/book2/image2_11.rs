#![allow(dead_code)]

use std::sync::Arc;

use crate::tools::bvh::BvhNode;
use crate::tools::camera::Camera;
use crate::tools::color::Color;
use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::hittable_list::HittableList;
use crate::tools::material::Lambertian;
use crate::tools::ray::Ray;
use crate::tools::render_utils::{create_progress_bar, prepare_output_path, save_image};
use crate::tools::rtweekend::INFINITY;
use crate::tools::sphere::Sphere;
use crate::tools::texture::NoiseTexture;
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
    let path = prepare_output_path("output/book2/image11.png");

    let pertext = Arc::new(NoiseTexture::new(1.0));
    let material = Arc::new(Lambertian::new_with_texture(pertext));
    let sphere1 = Box::new(Sphere::new_with_material(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        material.clone(),
    ));
    let sphere2 = Box::new(Sphere::new_with_material(
        Point3::new(0.0, 2.0, 0.0),
        2.0,
        material,
    ));
    let mut globe = HittableList::new();
    globe.add(sphere1);
    globe.add(sphere2);

    // 用 BVH 加速结构包装世界
    let bvh_node = BvhNode::from_list(globe);
    let mut globe = HittableList::new();
    globe.add(Box::new(bvh_node));

    // Camera
    let mut cam: Camera = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 400;
    cam.samples_per_pixel = 100;
    cam.max_depth = 50;

    cam.vfov = 20.0;
    cam.lookfrom = Point3::new(13.0, 2.0, 3.0);
    cam.lookat = Point3::new(0.0, 0.0, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0;
    cam.initialize();

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
                pixel_color += ray_color(&r, cam.max_depth, &globe);
            }
            let pixel = img.get_pixel_mut(i, j);
            Color::write_color_gamma(pixel_color * pixel_samples_scale, pixel);
            progress.inc(1);
        }
    }

    progress.finish();

    save_image(&img, &path);
}
