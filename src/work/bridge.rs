#![allow(dead_code)]

use std::sync::Arc;

use crate::tools::bvh::BvhNode;
use crate::tools::camera::Camera;
use crate::tools::color::Color;
use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::hittable_list::HittableList;
use crate::tools::material::{DiffuseLight, Metal};
use crate::tools::obj_loader::load_obj_transformed;
use crate::tools::quad::Quad;
use crate::tools::ray::Ray;
use crate::tools::render_utils::{
    create_progress_bar, prepare_output_path, render_parallel_gamma, save_image,
};
use crate::tools::rtweekend::INFINITY;
use crate::tools::sphere::Sphere;
use crate::tools::vector3::{Point3, Vec3, unit_vector};
use image::{ImageBuffer, RgbImage};

// ======================== 可调参数 ========================

/// 模型缩放因子（1.0 = 原始大小）
const SCALE: f64 = 0.5;

/// 绕 Y 轴旋转角度（度数，0 = 正面朝向 -Z 方向）
const ROTATE_Y_DEG: f64 = 0.0;

/// 模型平移向量（调整桥在场景中的位置）
const TRANSLATE: (f64, f64, f64) = (0.0, 0.0, 0.0);

// ======================== 渲染函数 ========================

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

    let color_from_scatter =
        attenuation * ray_color(&scattered, depth - 1, background, world);

    color_from_emission + color_from_scatter
}

/// 生成渐变天空背景色（深色调）
fn sky_background(r: &Ray) -> Color {
    let unit_direction = unit_vector(r.direction());
    let a = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - a) * Color::new(0.1, 0.1, 0.1) + a * Color::new(0.0, 0.0, 0.0)
}

pub fn render() {
    let path = prepare_output_path("output/work/bridge.png");

    // 加载桥模型
    let bridge_triangles = load_obj_transformed(
        "assets/bridge/source/Bridge-Section01.obj",
        Arc::new(Metal::new_with_fuzz(Color::new(0.75, 0.75, 0.78), 0.1)),
        SCALE,
        Vec3::new(TRANSLATE.0, TRANSLATE.1, TRANSLATE.2),
        ROTATE_Y_DEG,
    );

    // 构建世界
    let mut world = HittableList::new();

    // 添加桥梁三角形
    for tri in bridge_triangles {
        world.add(tri);
    }

    // 太阳：右上方橘黄色发光球体
    let sun = Arc::new(DiffuseLight::new(Color::new(10.0, 2.0, 0.5)));
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(-4.5, 3.5, 1.0),
        0.25,
        sun,
    )));

    // 水面：桥下方的大型反射平面
    let water_mat = Arc::new(Metal::new_with_fuzz(Color::new(0.08, 0.18, 0.30), 0.05));
    world.add(Box::new(Quad::new(
        Point3::new(-10.0, -0.5, -5.0),
        Vec3::new(20.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 10.0),
        water_mat,
    )));

    // BVH 加速
    let bvh_node = BvhNode::from_list(world);
    let mut world = HittableList::new();
    world.add(Box::new(bvh_node));

    // 相机
    let mut cam: Camera = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 1200;
    cam.samples_per_pixel = 100;
    cam.max_depth = 50;

    cam.vfov = 40.0;
    cam.lookfrom = Point3::new(0.0, 3.0, -8.0);
    cam.lookat = Point3::new(0.0, 1.5, 0.0);
    cam.vup = Vec3::new(0.0, 1.0, 0.0);

    cam.defocus_angle = 0.0;
    cam.focus_dist = 10.0;
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
                let bg = sky_background(&r);
                pixel_color += ray_color(&r, max_depth, &bg, &world);
            }
            pixel_color * pixel_samples_scale
        },
        &progress,
    );

    progress.finish();

    save_image(&img, &path);
}