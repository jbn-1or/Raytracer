#![allow(dead_code)]

use std::sync::Arc;

use crate::tools::bvh::BvhNode;
use crate::tools::camera::Camera;
use crate::tools::color::Color;
use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::hittable_list::HittableList;
use crate::tools::material::{Dielecric, Lambertian, Material, Metal};
use crate::tools::ray::Ray;
use crate::tools::render_utils::{create_progress_bar, prepare_output_path, save_image};
use crate::tools::rtweekend::{INFINITY, random_double, random_double_range};
use crate::tools::sphere::Sphere;
use crate::tools::texture::CheckerTexture;
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
    let path = prepare_output_path("output/book2/image2.png");

    // World
    let mut world: HittableList = HittableList::new();

    // 地面：棋盘纹理朗伯材质
    let checker_texture = Arc::new(CheckerTexture::from_another(
        0.32,
        Color::new(0.2, 0.3, 0.1),
        Color::new(0.9, 0.9, 0.9),
    ));
    let ground_material: Arc<dyn Material> =
        Arc::new(Lambertian::new_with_texture(checker_texture));
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        ground_material,
    )));

    // 随机生成小球
    for a in -11..11 {
        for b in -11..11 {
            let choose_mat = random_double();
            let center = Point3::new(
                a as f64 + 0.9 * random_double(),
                0.2,
                b as f64 + 0.9 * random_double(),
            );

            if (center - Point3::new(4.0, 0.2, 0.0)).length() > 0.9 {
                let sphere_material: Arc<dyn Material>;

                if choose_mat < 0.8 {
                    // diffuse
                    let albedo = Color::new(random_double(), random_double(), random_double())
                        * Color::new(random_double(), random_double(), random_double());
                    sphere_material = Arc::new(Lambertian::new(albedo));
                    let center2 = center + Vec3::new(0.0, random_double_range(0.0, 0.5), 0.0);
                    world.add(Box::new(Sphere::new_move_with_material(
                        center,
                        center2,
                        0.2,
                        sphere_material,
                    )));
                } else if choose_mat < 0.95 {
                    // metal
                    let albedo = Color::new(
                        random_double_range(0.5, 1.0),
                        random_double_range(0.5, 1.0),
                        random_double_range(0.5, 1.0),
                    );
                    let fuzz = random_double_range(0.0, 0.5);
                    sphere_material = Arc::new(Metal::new_with_fuzz(albedo, fuzz));
                    world.add(Box::new(Sphere::new_with_material(
                        center,
                        0.2,
                        sphere_material,
                    )));
                } else {
                    // glass
                    sphere_material = Arc::new(Dielecric::new(1.5));
                    world.add(Box::new(Sphere::new_with_material(
                        center,
                        0.2,
                        sphere_material,
                    )));
                }
            }
        }
    }

    // 三个大球
    let material1: Arc<dyn Material> = Arc::new(Dielecric::new(1.5));
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(0.0, 1.0, 0.0),
        1.0,
        material1,
    )));

    let material2: Arc<dyn Material> = Arc::new(Lambertian::new(Color::new(0.4, 0.2, 0.1)));
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(-4.0, 1.0, 0.0),
        1.0,
        material2,
    )));

    let material3: Arc<dyn Material> =
        Arc::new(Metal::new_with_fuzz(Color::new(0.7, 0.6, 0.5), 0.0));
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(4.0, 1.0, 0.0),
        1.0,
        material3,
    )));

    // 用 BVH 加速结构包装世界
    let bvh_node = BvhNode::from_list(world);
    let mut world = HittableList::new();
    world.add(Box::new(bvh_node));

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

    cam.defocus_angle = 0.6;
    cam.focus_dist = 10.0;
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
                pixel_color += ray_color(&r, cam.max_depth, &world);
            }
            let pixel = img.get_pixel_mut(i, j);
            Color::write_color_gamma(pixel_color * pixel_samples_scale, pixel);
            progress.inc(1);
        }
    }

    progress.finish();

    save_image(&img, &path);
}
