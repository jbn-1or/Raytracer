#![allow(dead_code)]

use std::sync::Arc;

use crate::tools::camera::Camera;
use crate::tools::color::Color;
use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::hittable_list::HittableList;
use crate::tools::material::{DiffuseLight, Lambertian};
use crate::tools::quad::Quad;
use crate::tools::ray::Ray;
use crate::tools::render_utils::{create_progress_bar, prepare_output_path, save_image};
use crate::tools::rtweekend::INFINITY;
use crate::tools::sphere::Sphere;
use crate::tools::texture::NoiseTexture;
use crate::tools::vector3::{Point3, Vec3};
use image::{ImageBuffer, RgbImage};

fn ray_color(r: &Ray, depth: u32, background: &Color, world: &dyn Hittable) -> Color {
    // 如果超过光线反弹限制，不再收集光线
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    let mut rec: HitRecord = HitRecord::default();

    // 如果光线未击中任何物体，返回背景色
    if !world.hit(r, 0.001, INFINITY, &mut rec) {
        return *background;
    }

    let mut scattered: Ray = Ray::zero();
    let mut attenuation: Color = Color::zero();

    let color_from_emission = if let Some(ref mat) = rec.mat {
        mat.emitted(rec.u, rec.v, rec.p)
    } else {
        Color::new(0.0, 0.0, 0.0)
    };

    if let Some(ref mat) = rec.mat {
        if !mat.scatter(r, &rec, &mut attenuation, &mut scattered) {
            return color_from_emission;
        }
    } else {
        return color_from_emission;
    }

    let color_from_scatter = attenuation * ray_color(&scattered, depth - 1, background, world);

    color_from_emission + color_from_scatter
}

#[allow(non_snake_case)]
pub fn render() {
    let path = prepare_output_path("output/book2/image17.png");

    // World
    let mut world = HittableList::new();

    let pertext = Arc::new(NoiseTexture::new(4.0));
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(0.0, -1000.0, 0.0),
        1000.0,
        Arc::new(Lambertian::new_with_texture(pertext.clone())),
    )));
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(0.0, 2.0, 0.0),
        2.0,
        Arc::new(Lambertian::new_with_texture(pertext)),
    )));

    let difflight = Arc::new(DiffuseLight::new(Color::new(4.0, 4.0, 4.0)));
    world.add(Box::new(Quad::new(
        Point3::new(3.0, 1.0, -2.0),
        Vec3::new(2.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        difflight,
    )));

    // Camera
    let mut cam: Camera = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 400;
    cam.samples_per_pixel = 100;
    cam.max_depth = 50;
    cam.background = Color::new(0.0, 0.0, 0.0);

    cam.vfov = 20.0;
    cam.lookfrom = Point3::new(26.0, 3.0, 6.0);
    cam.lookat = Point3::new(0.0, 2.0, 0.0);
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
                pixel_color += ray_color(&r, cam.max_depth, &cam.background, &world);
            }
            let pixel = img.get_pixel_mut(i, j);
            Color::write_color_gamma(pixel_color * pixel_samples_scale, pixel);
            progress.inc(1);
        }
    }

    progress.finish();

    save_image(&img, &path);
}
