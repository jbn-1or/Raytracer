#![allow(dead_code)]

use std::sync::Arc;

use crate::tools::bvh::BvhNode;
use crate::tools::camera::Camera;
use crate::tools::color::Color;
use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::hittable_list::HittableList;
use crate::tools::material::{DiffuseLight, GlassWithBlackCore, Lambertian, Material, WaterMetal};
use crate::tools::quad::Quad;
use crate::tools::ray::Ray;
use crate::tools::render_utils::{
    create_progress_bar, prepare_output_path, render_parallel_gamma, save_image,
};
use crate::tools::rtweekend::INFINITY;
use crate::tools::sphere::Sphere;
use crate::tools::texture::ImageTexture;
use crate::tools::triangle::Triangle;
use crate::tools::vector3::{Point3, Vec3};
use image::{ImageBuffer, RgbImage};

/// 图像宽度（16:9）
const IMAGE_WIDTH: u32 = 5760;

/// 每像素采样数
const SAMPLES: u32 = 800;

/// 最大反弹次数
const MAX_DEPTH: u32 = 20;

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

/// 纯黑背景
fn black_sky(_r: &Ray) -> Color {
    Color::new(0.0, 0.0, 0.0)
}

// ======================== 场景构建 ========================

pub fn render() {
    let path = prepare_output_path("output/work/final_scene.png");

    // ============ 材质 ============

    // 照片发光平面（NASA 全球夜光图）
    let night_tex = Arc::new(ImageTexture::new("images/earth_night.jpg"));
    let light_mat = Arc::new(DiffuseLight::new_with_texture(night_tex));

    // 波纹水面
    let water_surface = Arc::new(WaterMetal::new(
        Color::new(1.0, 0.8, 0.8),
        8.0,  // 波纹空间频率
        0.15, // 波纹强度
    ));

    // 光源
    let light = Arc::new(DiffuseLight::new(Color::new(1.0, 1.0, 1.0)));

    // 星球
    let planet_texture1 = Arc::new(ImageTexture::new("images/planet/2k_jupiter.jpg"));
    let planet_mat1 = Arc::new(Lambertian::new_with_texture(planet_texture1));

    let planet_texture2 = Arc::new(ImageTexture::new("images/planet/2k_mercury.jpg"));
    let planet_mat2 = Arc::new(Lambertian::new_with_texture(planet_texture2));

    // 海王星
    let neptune_tex = Arc::new(ImageTexture::new("images/planet/2k_neptune.jpg"));
    let neptune_mat = Arc::new(Lambertian::new_with_texture(neptune_tex));
    // 黑色车漆（暗灰内核，高折射率增强反射）
    let black_paint = Arc::new(GlassWithBlackCore::new_with_core(
        1.2,
        Color::new(0.8, 0.04, 0.04),
    ));

    // ============ 世界 ============
    let mut world = HittableList::new();

    // ---- 发光照片平面（正面朝向相机） ----
    world.add(Box::new(Quad::new(
        Point3::new(-1.9, -1.6, 0.0),
        Vec3::new(4.0, 0.0, 0.0),
        Vec3::new(0.0, 2.0, 0.0),
        light_mat,
    )));

    // 波纹水面（大平面镜）
    world.add(Box::new(Quad::new(
        Point3::new(-10.0, -1.0, -10.0),
        Vec3::new(20.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 20.0),
        water_surface,
    )));

    // 海王星
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(-0.3, -0.97, 2.0),
        0.03,
        neptune_mat,
    )));

    // 星球
    world.add(Box::new(Sphere::new_with_material(
        Point3::new(1.0, -0.80, 1.0),
        0.12,
        planet_mat1,
    )));

    // 月球 - 带向左的运动模糊
    world.add(Box::new(Sphere::new_move_with_material(
        Point3::new(0.6, -0.95, 1.5),
        Point3::new(0.35, -0.95, 1.5),
        0.05,
        planet_mat2,
    )));

    // 发光平面1
    world.add(Box::new(Quad::new(
        Point3::new(-10.0, 1.0, -10.0),
        Vec3::new(20.0, 0.0, 0.0),
        Vec3::new(0.0, 0.0, 20.0),
        light.clone(),
    )));

    // 发光平面2
    world.add(Box::new(Quad::new(
        Point3::new(-10.0, -10.0, 5.0),
        Vec3::new(20.0, 0.0, 0.0),
        Vec3::new(0.0, 20.0, 0.0),
        light,
    )));

    // ---- 跑车模型 ----
    // 原始尺寸：X[-3.81, 3.81], Y[-0.93, 2.67], Z[-6.96, 6.94]
    println!("Loading car model...");
    let car_triangles = load_obj_ignore_normals(
        "assets/sportcar.obj",
        black_paint,
        0.065,
        Vec3::new(0.0, -0.945, 1.0),
        -55.0,
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

    // 相机正对照片平面中心（保持 earth_night 原有设置）
    cam.lookfrom = Point3::new(-0.1, -0.85, 3.2);
    cam.lookat = Point3::new(-0.1, -0.65, 0.0);
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
    println!("Done! Output: {:?}", path);
}
