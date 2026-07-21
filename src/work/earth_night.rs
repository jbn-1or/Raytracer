#![allow(dead_code)]

use std::sync::Arc;

use crate::tools::camera::Camera;
use crate::tools::color::Color;
use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::hittable_list::HittableList;
use crate::tools::material::{DiffuseLight, Metal};
use crate::tools::quad::Quad;
use crate::tools::ray::Ray;
use crate::tools::render_utils::{
    create_progress_bar, prepare_output_path, render_parallel_gamma, save_image,
};
use crate::tools::rtweekend::INFINITY;
use crate::tools::texture::ImageTexture;
use crate::tools::vector3::{Point3, Vec3};
use image::{ImageBuffer, RgbImage};

/// 图像宽度（16:9）
const IMAGE_WIDTH: u32 = 1600;

/// 每像素采样数
const SAMPLES: u32 = 200;

/// 最大反弹次数
const MAX_DEPTH: u32 = 20;

/// 视野角度
const VFOV: f64 = 40.0;

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

/// 纯黑背景
fn black_sky(_r: &Ray) -> Color {
    Color::new(0.0, 0.0, 0.0)
}

// ======================== 场景构建 ========================

pub fn render() {
    let path = prepare_output_path("output/work/earth_night.png");

    // ============ 材质 ============

    // 照片发光平面（NASA 全球夜光图）
    let night_tex = Arc::new(ImageTexture::new("images/earth_night.jpg"));
    let light_mat = Arc::new(DiffuseLight::new_with_texture(night_tex));
    let water_surface = Arc::new(Metal::new(Color::new(1.0, 1.0, 1.0)));

    // ============ 世界 ============
    let mut world = HittableList::new();

    // ---- 发光照片平面（正面朝向相机） ----
    world.add(Box::new(Quad::new(
        Point3::new(-3.0, -1.5, 0.0),
        Vec3::new(6.0, 0.0, 0.0),
        Vec3::new(0.0, 3.0, 0.0),
        light_mat,
    )));

    // 大平面镜（水面）
    world.add(Box::new(Quad::new(
        Point3::new(-10.0, -1.0, -10.0),
        Vec3::new(20.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 20.0),
        water_surface,
    )));

    // ============ 相机 ============
    let mut cam: Camera = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = IMAGE_WIDTH;
    cam.samples_per_pixel = SAMPLES;
    cam.max_depth = MAX_DEPTH;

    cam.vfov = VFOV;

    // 相机正对照片平面中心
    cam.lookfrom = Point3::new(0.0, 0.0, 5.0);
    cam.lookat = Point3::new(0.0, 0.0, 0.0);
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
                let bg = black_sky(&r);
                pixel_color += ray_color(&r, max_depth, &bg, &world);
            }
            pixel_color * pixel_samples_scale
        },
        &progress,
    );

    progress.finish();

    save_image(&img, &path);
}
