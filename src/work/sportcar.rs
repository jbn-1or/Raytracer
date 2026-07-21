#![allow(dead_code)]

use std::sync::Arc;

use crate::tools::bvh::BvhNode;
use crate::tools::camera::Camera;
use crate::tools::color::Color;
use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::hittable_list::HittableList;
use crate::tools::material::{DiffuseLight, GlassWithBlackCore, Lambertian, Material};
use crate::tools::quad::Quad;
use crate::tools::ray::Ray;
use crate::tools::render_utils::{
    create_progress_bar, prepare_output_path, render_parallel_gamma, save_image,
};
use crate::tools::rtweekend::INFINITY;
use crate::tools::sphere::Sphere;
use crate::tools::triangle::Triangle;
use crate::tools::vector3::{Point3, Vec3, unit_vector};
use image::{ImageBuffer, RgbImage};

/// 图像宽度
const IMAGE_WIDTH: u32 = 800;

/// 每像素采样数
const SAMPLES: u32 = 100;

/// 最大反弹次数
const MAX_DEPTH: u32 = 10;

/// 视野角度
const VFOV: f64 = 30.0;

/// 加载 OBJ 并变换（忽略法线，避免法线/顶点索引不匹配导致的越界）
fn load_obj_ignore_normals(
    path: &str,
    mat: Arc<dyn Material>,
    scale: f64,
    translate: Vec3,
    rotate_y_deg: f64,
) -> Vec<Box<dyn Hittable>> {
    let (models, _materials) = tobj::load_obj(
        path,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
    )
    .unwrap_or_else(|_| panic!("Failed to load OBJ file: {}", path));

    let radians = rotate_y_deg.to_radians();
    let sin_theta = radians.sin();
    let cos_theta = radians.cos();

    let mut triangles: Vec<Box<dyn Hittable>> = Vec::new();

    for model in models.iter() {
        let mesh = &model.mesh;

        let raw_positions: Vec<Point3> = mesh
            .positions
            .chunks(3)
            .map(|p| Point3::new(p[0] as f64, p[1] as f64, p[2] as f64))
            .collect();

        let positions: Vec<Point3> = raw_positions
            .iter()
            .map(|p| {
                let sx = p.x() * scale;
                let sy = p.y() * scale;
                let sz = p.z() * scale;
                let rx = cos_theta * sx + sin_theta * sz;
                let ry = sy;
                let rz = -sin_theta * sx + cos_theta * sz;
                Point3::new(rx + translate.x(), ry + translate.y(), rz + translate.z())
            })
            .collect();

        for face in mesh.indices.chunks(3) {
            if face.len() < 3 {
                continue;
            }

            let i0 = face[0] as usize;
            let i1 = face[1] as usize;
            let i2 = face[2] as usize;

            if i0 >= positions.len() || i1 >= positions.len() || i2 >= positions.len() {
                continue;
            }

            let v0 = positions[i0];
            let v1 = positions[i1];
            let v2 = positions[i2];

            let tri = Triangle::new(v0, v1, v2);
            triangles.push(Box::new(tri.with_material(mat.clone())));
        }
    }

    triangles
}

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
    let path = prepare_output_path("output/work/sportcar.png");

    // ============ 材质 ============

    // 黑色车漆（暗灰内核，微微有层次感）
    let black_paint = Arc::new(GlassWithBlackCore::new_with_core(
        1.5,
        Color::new(0.04, 0.04, 0.04),
    ));

    // 地面材质
    let ground_mat = Arc::new(Lambertian::new(Color::new(0.5, 0.5, 0.55)));

    // 顶部矩形面光源
    let light_mat = Arc::new(DiffuseLight::new(Color::new(3.0, 3.0, 3.0)));

    // ============ 世界 ============
    let mut world = HittableList::new();

    // 大地面
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(0.0, -1000.5, 0.0),
        1000.0,
        ground_mat,
    )));

    // 顶部面光源
    world.add(Box::new(Quad::new(
        Point3::new(-3.0, 6.0, -4.0),
        Vec3::new(6.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 4.0),
        light_mat,
    )));

    // 加载跑车模型并用 BVH 加速
    // 原始尺寸：X[-3.81, 3.81], Y[-0.93, 2.67], Z[-6.96, 6.94], 中心(0.02, 0.45, -1.03)
    // 缩放 0.35 → 约 5 单位长，Y 方向约 0.8 单位高
    // 绕 Y 旋转 15 度展示侧面，下移使车轮着地
    println!("Loading car model...");
    let car_triangles = load_obj_ignore_normals(
        "assets/sportcar.obj",
        black_paint,
        0.35,
        Vec3::new(0.0, -0.45 * 0.35, -2.0),
        15.0,
    );
    println!("Loaded {} triangles, building BVH...", car_triangles.len());

    // 将跑车三角形放入独立的 HittableList，然后构建 BVH
    let mut car_list = HittableList::new();
    for tri in car_triangles {
        car_list.add(tri);
    }
    let car_bvh = BvhNode::from_list(car_list);
    world.add(Box::new(car_bvh));
    println!("BVH built.");

    // ============ 相机 ============
    let mut cam: Camera = Camera::new();
    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = IMAGE_WIDTH;
    cam.samples_per_pixel = SAMPLES;
    cam.max_depth = MAX_DEPTH;

    cam.vfov = VFOV;

    // 从前方略高处看向车身
    cam.lookfrom = Point3::new(3.0, 1.5, 5.0);
    cam.lookat = Point3::new(0.0, -0.2, -2.0);
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
    println!("Done! Output: {:?}", path);
}
