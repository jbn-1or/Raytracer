#![allow(dead_code)]

use std::sync::Arc;

use crate::tools::bvh::BvhNode;
use crate::tools::camera::Camera;
use crate::tools::color::Color;
use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::hittable_list::HittableList;
use crate::tools::material::{Dielecric, DiffuseLight, Metal};
use crate::tools::quad::Quad;
use crate::tools::ray::Ray;
use crate::tools::render_utils::{
    create_progress_bar, prepare_output_path, render_parallel_gamma, save_image,
};
use crate::tools::rtweekend::INFINITY;
use crate::tools::sphere::Sphere;
use crate::tools::texture::CheckerTexture;
use crate::tools::vector3::{Point3, Vec3, unit_vector};
use image::{ImageBuffer, RgbImage};

/// 图像宽度（16:9）
const IMAGE_WIDTH: u32 = 800;

/// 每像素采样数
const SAMPLES: u32 = 200;

/// 最大反弹次数
const MAX_DEPTH: u32 = 100;

/// 视野角度
const VFOV: f64 = 90.0;

/// 棋盘格地面缩放
const CHECKER_SCALE: f64 = 1.0;

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

/// 月夜天空背景：深蓝渐变到黑
fn moonlit_sky(r: &Ray) -> Color {
    let unit_direction = unit_vector(r.direction());
    let a = 0.5 * (unit_direction.y() + 1.0);
    // 深靛蓝 -> 纯黑
    let sky_top = Color::new(0.02, 0.02, 0.10);
    let sky_horizon = Color::new(0.03, 0.05, 0.15);
    (1.0 - a) * sky_horizon + a * sky_top
}

// ======================== 场景构建 ========================

pub fn render() {
    let path = prepare_output_path("output/work/ball.png");

    // ============ 材质 ============

    // 棋盘格地板
    let checker = Arc::new(CheckerTexture::from_another(
        CHECKER_SCALE,
        Color::new(1.0, 1.0, 1.0),
        Color::new(1.0, 1.0, 1.0),
    ));
    let floor_mat = Arc::new(DiffuseLight::new_with_texture(checker));
    let mirror = Arc::new(Metal::new_with_fuzz(Color::new(0.8, 0.8, 0.8), 0.0));
    let ball = Arc::new(Dielecric::new(1.5));

    // 天空
    // let warm_light = Arc::new(DiffuseLight::new(Color::new(1.0, 1.0, 1.0)));

    // 月光（冷蓝白）
    // let moon_light = Arc::new(DiffuseLight::new(Color::new(0.5, 1.5, 8.0)));

    // ============ 世界 ============
    let mut world = HittableList::new();

    // ---- 地面（大型棋盘格平面） ----
    world.add(Box::new(Quad::new(
        Point3::new(-50.0, -50.0, 0.0),
        Vec3::new(100.0, 0.0, 0.0),
        Vec3::new(0.0, 100.0, 0.0),
        floor_mat,
    )));

    // ---- 玻璃球
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(0.2, 0.2, 0.5),
        0.3,
        ball,
    )));

    // ---- 朗伯球

    // 镜子
    world.add(Box::new(Quad::new(
        Point3::new(
            1.5 * CHECKER_SCALE,
            1.5 * CHECKER_SCALE,
            0.0 * CHECKER_SCALE,
        ),
        Vec3::new(
            0.0 * CHECKER_SCALE,
            -3.0 * CHECKER_SCALE,
            0.0 * CHECKER_SCALE,
        ),
        Vec3::new(0.0, 0.0, 10.0 * CHECKER_SCALE),
        mirror.clone(),
    )));

    world.add(Box::new(Quad::new(
        Point3::new(
            1.5 * CHECKER_SCALE,
            1.5 * CHECKER_SCALE,
            0.0 * CHECKER_SCALE,
        ),
        Vec3::new(
            -3.0 * CHECKER_SCALE,
            0.0 * CHECKER_SCALE,
            0.0 * CHECKER_SCALE,
        ),
        Vec3::new(0.0, 0.0, 10.0 * CHECKER_SCALE),
        mirror.clone(),
    )));

    world.add(Box::new(Quad::new(
        Point3::new(
            -1.5 * CHECKER_SCALE,
            -1.5 * CHECKER_SCALE,
            0.0 * CHECKER_SCALE,
        ),
        Vec3::new(
            3.0 * CHECKER_SCALE,
            0.0 * CHECKER_SCALE,
            0.0 * CHECKER_SCALE,
        ),
        Vec3::new(0.0, 0.0, 10.0 * CHECKER_SCALE),
        mirror.clone(),
    )));

    world.add(Box::new(Quad::new(
        Point3::new(
            -1.5 * CHECKER_SCALE,
            -1.5 * CHECKER_SCALE,
            0.0 * CHECKER_SCALE,
        ),
        Vec3::new(
            0.0 * CHECKER_SCALE,
            3.0 * CHECKER_SCALE,
            0.0 * CHECKER_SCALE,
        ),
        Vec3::new(0.0, 0.0, 10.0 * CHECKER_SCALE),
        mirror,
    )));

    // ============ BVH 加速 ============
    let bvh_node = BvhNode::from_list(world);
    let mut world = HittableList::new();
    world.add(Box::new(bvh_node));

    // ============ 相机 ============
    let mut cam: Camera = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = IMAGE_WIDTH;
    cam.samples_per_pixel = SAMPLES;
    cam.max_depth = MAX_DEPTH;

    cam.vfov = VFOV;

    cam.lookfrom = Point3::new(-1.0, -1.00, 1.0);
    cam.lookat = Point3::new(0.0, 0.0, 0.0);
    cam.vup = Vec3::new(0.0, 0.0, 1.0);

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
                let bg = moonlit_sky(&r);
                pixel_color += ray_color(&r, max_depth, &bg, &world);
            }
            pixel_color * pixel_samples_scale
        },
        &progress,
    );

    progress.finish();

    save_image(&img, &path);
}
