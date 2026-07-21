#![allow(dead_code)]

use std::sync::Arc;

use crate::tools::camera::Camera;
use crate::tools::color::Color;
use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::hittable_list::HittableList;
use crate::tools::material::{DiffuseLight, GlassWithBlackCore, Lambertian, Metal};
use crate::tools::quad::Quad;
use crate::tools::ray::Ray;
use crate::tools::render_utils::{
    create_progress_bar, prepare_output_path, render_parallel_gamma, save_image,
};
use crate::tools::rtweekend::INFINITY;
use crate::tools::sphere::Sphere;
use crate::tools::vector3::{Point3, Vec3, unit_vector};
use image::{ImageBuffer, RgbImage};

/// 图像宽度（16:9）
const IMAGE_WIDTH: u32 = 400;

/// 每像素采样数
const SAMPLES: u32 = 50;

/// 最大反弹次数
const MAX_DEPTH: u32 = 20;

/// 视野角度
const VFOV: f64 = 30.0;

// ======================== 光线追踪核心 ========================

fn ray_color(r: &Ray, depth: u32, background: &Color, world: &dyn Hittable) -> Color {
    if depth == 0 {
        return Color::new(0.0, 0.0, 0.0);
    }

    let mut rec: HitRecord = HitRecord::default();

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

/// 渐变背景（白到蓝）
fn gradient_sky(r: &Ray) -> Color {
    let unit = unit_vector(r.direction());
    let a = 0.5 * (unit.y() + 1.0);
    Color::new(1.0, 1.0, 1.0) * (1.0 - a) + Color::new(0.5, 0.7, 1.0) * a
}

// ======================== 场景构建 ========================

pub fn render() {
    let path = prepare_output_path("output/work/black_car_paint.png");

    // ============ 材质 ============

    // 纯黑内核的玻璃（黑色车漆）
    let black_paint = Arc::new(GlassWithBlackCore::new(1.5));

    // 暗灰色内核的玻璃（微微有点散射的黑色车漆）
    let black_paint_soft = Arc::new(GlassWithBlackCore::new_with_core(
        1.5,
        Color::new(0.05, 0.05, 0.05),
    ));

    // 灰色朗伯体作为对比
    let gray_lambert = Arc::new(Lambertian::new(Color::new(0.3, 0.3, 0.3)));

    // 金属球作为对比
    let metal_chrome = Arc::new(Metal::new(Color::new(0.9, 0.9, 0.9)));

    // 顶部光源
    let light_mat = Arc::new(DiffuseLight::new(Color::new(4.0, 4.0, 4.0)));

    // 地面材质
    let ground_mat = Arc::new(Lambertian::new(Color::new(0.8, 0.8, 0.8)));

    // ============ 世界 ============
    let mut world = HittableList::new();

    // 大地面
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(0.0, -100.5, -2.0),
        100.0,
        ground_mat,
    )));

    // 一圈展示球体
    // 黑色车漆球（纯黑内核）
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(-1.8, -0.5, -2.0),
        1.0,
        black_paint.clone(),
    )));

    // 黑色车漆球（暗灰内核，更有层次感）
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(0.0, -0.5, -2.0),
        1.0,
        black_paint_soft,
    )));

    // 灰色朗伯体对比球
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(1.8, -0.5, -2.0),
        1.0,
        gray_lambert,
    )));

    // 金属对比球
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(3.6, -0.5, -2.0),
        1.0,
        metal_chrome,
    )));

    // 顶部矩形面光源
    world.add(Box::new(Quad::new(
        Point3::new(-2.0, 6.0, -4.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 2.0),
        light_mat,
    )));

    // ============ 相机 ============
    let mut cam: Camera = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = IMAGE_WIDTH;
    cam.samples_per_pixel = SAMPLES;
    cam.max_depth = MAX_DEPTH;

    cam.vfov = VFOV;

    cam.lookfrom = Point3::new(2.0, 2.0, 5.0);
    cam.lookat = Point3::new(0.0, -0.5, -2.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0;
    cam.initialize();

    let image_width = cam.image_width;
    let image_height = cam.image_height();
    let pixel_samples_scale = cam.pixel_samples_scale();

    let mut img: RgbImage = ImageBuffer::new(image_width, image_height);

    let progress = create_progress_bar((image_height * image_width) as u64);

    let samples = cam.samples_per_pixel;
    let max_depth = cam.max_depth;

    render_parallel_gamma(
        &mut img,
        image_width,
        image_height,
        move |i, j| {
            let mut pixel_color: Color = Color::zero();
            for _sample in 0..samples {
                let r = cam.get_ray(i, j);
                let bg = gradient_sky(&r);
                pixel_color += ray_color(&r, max_depth, &bg, &world);
            }
            pixel_color * pixel_samples_scale
        },
        &progress,
    );

    progress.finish();

    save_image(&img, &path);
}
