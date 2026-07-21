#![allow(dead_code)]

use std::sync::Arc;

use crate::tools::bvh::BvhNode;
use crate::tools::camera::Camera;
use crate::tools::color::Color;
use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::hittable_list::HittableList;
use crate::tools::material::{Dielecric, DiffuseLight, Lambertian, Metal};
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

// ======================== 可调参数 ========================

/// 图像宽度（16:9）
const IMAGE_WIDTH: u32 = 1200;

/// 每像素采样数
const SAMPLES: u32 = 200;

/// 最大反弹次数
const MAX_DEPTH: u32 = 50;

/// 视野角度
const VFOV: f64 = 30.0;

/// 棋盘格地面缩放
const CHECKER_SCALE: f64 = 0.8;

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
    let path = prepare_output_path("output/work/glass_tower.png");

    // ============ 材质 ============

    // 棋盘格地板：深灰 + 浅灰，模拟大理石
    let checker = Arc::new(CheckerTexture::from_another(
        CHECKER_SCALE,
        Color::new(0.05, 0.05, 0.07),
        Color::new(0.15, 0.15, 0.18),
    ));
    let floor_mat = Arc::new(Lambertian::new_with_texture(checker));

    // 底部暖光面（橙金色）
    let warm_light = Arc::new(DiffuseLight::new(Color::new(6.0, 2.0, 0.5)));

    // 月光（冷蓝白）
    let moon_light = Arc::new(DiffuseLight::new(Color::new(0.5, 1.5, 4.0)));

    // 玻璃折射率
    let glass_heavy = Arc::new(Dielecric::new(1.58)); // 高折射（火石玻璃）
    let glass_normal = Arc::new(Dielecric::new(1.50)); // 标准玻璃
    let glass_light = Arc::new(Dielecric::new(1.33)); // 低折射（水/冰感）

    // 金属点缀球
    let gold = Arc::new(Metal::new_with_fuzz(Color::new(0.83, 0.69, 0.22), 0.05));
    let silver = Arc::new(Metal::new_with_fuzz(Color::new(0.75, 0.75, 0.78), 0.03));
    let copper = Arc::new(Metal::new_with_fuzz(Color::new(0.72, 0.45, 0.20), 0.08));

    // 底部发光台座（漫反射环）
    let pedestal = Arc::new(Lambertian::new(Color::new(0.12, 0.12, 0.14)));

    // ============ 世界 ============
    let mut world = HittableList::new();

    // ---- 地面（大型棋盘格平面） ----
    world.add(Box::new(Quad::new(
        Point3::new(-50.0, -0.01, -50.0),
        Vec3::new(100.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 100.0),
        floor_mat,
    )));

    // ---- 底部暖光面（塔正下方的发光方块） ----
    world.add(Box::new(Quad::new(
        Point3::new(-1.2, 0.005, -1.8),
        Vec3::new(2.4, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 3.6),
        warm_light.clone(),
    )));

    // ---- 台座环（漫反射圆柱模拟，用 4 个 Quad 围成） ----
    // 前侧
    world.add(Box::new(Quad::new(
        Point3::new(-1.5, 0.0, 2.0),
        Vec3::new(3.0, 0.0, 0.0),
        Vec3::new(0.0, 0.5, 0.0),
        pedestal.clone(),
    )));
    // 后侧
    world.add(Box::new(Quad::new(
        Point3::new(-1.5, 0.0, -2.0),
        Vec3::new(3.0, 0.0, 0.0),
        Vec3::new(0.0, 0.5, 0.0),
        pedestal.clone(),
    )));
    // 左侧
    world.add(Box::new(Quad::new(
        Point3::new(-1.5, 0.0, -2.0),
        Vec3::new(0.0, 0.0, 4.0),
        Vec3::new(0.0, 0.5, 0.0),
        pedestal.clone(),
    )));
    // 右侧
    world.add(Box::new(Quad::new(
        Point3::new(1.5, 0.0, -2.0),
        Vec3::new(0.0, 0.0, 4.0),
        Vec3::new(0.0, 0.5, 0.0),
        pedestal,
    )));

    // ---- 月光球（右上方大发光球） ----
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(3.0, 2.0, -1.5),
        0.6,
        moon_light,
    )));

    // ---- 玻璃塔：3 层球体竖直堆叠（中心 x=0, z=0） ----
    // 底层大球（高折射）
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(0.0, 0.7, 0.0),
        0.55,
        glass_heavy.clone(),
    )));
    // 中层球（标准折射，略偏置）
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(0.0, 1.6, 0.0),
        0.45,
        glass_normal.clone(),
    )));
    // 顶层小球（低折射，冰感）
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(0.0, 2.4, 0.0),
        0.32,
        glass_light.clone(),
    )));

    // ---- 散布的小玻璃珠 ----
    // 塔左侧
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(-0.9, 0.55, 0.4),
        0.12,
        glass_normal.clone(),
    )));
    // 塔右侧
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(0.85, 0.60, -0.3),
        0.15,
        glass_light.clone(),
    )));
    // 塔前方
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(0.0, 0.50, 1.0),
        0.10,
        glass_heavy.clone(),
    )));

    // ---- 金属点缀球 ----
    // 金球（左前）
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(-1.8, 0.35, 1.5),
        0.20,
        gold,
    )));
    // 银球（右前）
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(1.7, 0.30, 1.2),
        0.18,
        silver,
    )));
    // 铜球（正前方稍远）
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(0.3, 0.30, 2.2),
        0.25,
        copper,
    )));
    // 小金球（左后方）
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(-2.2, 0.18, -0.5),
        0.10,
        Arc::new(Metal::new_with_fuzz(Color::new(0.83, 0.69, 0.22), 0.02)),
    )));
    // 银球（右后方）
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(2.0, 0.15, -0.8),
        0.12,
        Arc::new(Metal::new_with_fuzz(Color::new(0.75, 0.75, 0.78), 0.02)),
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
    // 低角度仰拍，突出高耸感
    cam.lookfrom = Point3::new(2.8, 1.2, -6.5);
    cam.lookat = Point3::new(0.0, 1.5, 0.0);
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
