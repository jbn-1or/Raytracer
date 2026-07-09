#![allow(dead_code)]

use std::path::Path;

use console::style;
use image::{ImageBuffer, RgbImage};
use indicatif::ProgressBar;

use super::color::Color;
use super::hittable::{HitRecord, Hittable};
use super::ray::Ray;
use super::rtweekend::{INFINITY, random_double};
use super::vector3::{Point3, Vec3, unit_vector};

/// 相机类，负责构造并向场景分发光线，并利用光线结果构建渲染图像
pub struct Camera {
    /// 图像宽高比
    pub aspect_ratio: f64,
    /// 渲染图像宽度（像素）
    pub image_width: u32,
    /// 每像素发射多条光线并对颜色取平均
    pub samples_per_pixel: u32,

    /// 渲染图像高度
    image_height: u32,

    pixel_samples_scale: f64,
    /// 相机中心位置
    center: Point3,
    /// 像素 (0, 0) 的世界坐标
    pixel00_loc: Point3,
    /// 水平方向相邻像素的偏移向量
    pixel_delta_u: Vec3,
    /// 垂直方向相邻像素的偏移向量
    pixel_delta_v: Vec3,
}

impl Camera {
    /// 创建一个使用默认参数的相机
    pub fn new() -> Self {
        Self {
            aspect_ratio: 1.0,
            image_width: 100,
            samples_per_pixel: 10,
            image_height: 0,
            pixel_samples_scale: 0.1,
            center: Point3::zero(),
            pixel00_loc: Point3::zero(),
            pixel_delta_u: Vec3::zero(),
            pixel_delta_v: Vec3::zero(),
        }
    }

    /// 渲染场景并将结果保存为 PNG 图像
    /// # 参数
    /// - `world`-可击中物体列表的 trait 引用
    /// - `path`-输出 PNG 文件的路径
    pub fn render<P: AsRef<Path>>(&mut self, world: &dyn Hittable, path: P) {
        self.initialize();

        let prefix = path.as_ref().parent().unwrap();
        std::fs::create_dir_all(prefix).expect("Cannot create output directory");

        let mut img: RgbImage = ImageBuffer::new(self.image_width, self.image_height);

        let progress = if option_env!("CI").unwrap_or_default() == "true" {
            ProgressBar::hidden()
        } else {
            ProgressBar::new((self.image_height * self.image_width) as u64)
        };

        for j in 0..self.image_height {
            for i in 0..self.image_width {
                let mut pixel_color: Color = Color::zero();
                for _sample in 0..self.samples_per_pixel {
                    let r = self.get_ray(i, j);
                    pixel_color += self.ray_color(&r, world);
                }
                let pixel = img.get_pixel_mut(i, j);
                Color::write_color(pixel_color * self.pixel_samples_scale, pixel);
                progress.inc(1);
            }
        }

        progress.finish();

        println!(
            "Output image as \"{}\"",
            style(path.as_ref().to_str().unwrap()).yellow()
        );
        img.save(path).expect("Cannot save the image to the file");
    }

    /// 初始化相机参数：计算图像高度、视口尺寸、像素偏移等
    fn initialize(&mut self) {
        self.image_height = (self.image_width as f64 / self.aspect_ratio) as u32;
        if self.image_height < 1 {
            self.image_height = 1;
        }

        self.pixel_samples_scale = 1.0 / self.samples_per_pixel as f64;

        self.center = Point3::new(0.0, 0.0, 0.0);

        // 确定视口尺寸
        let focal_length = 1.0;
        let viewport_height = 2.0;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        // 计算水平与垂直方向跨越视口边缘的向量
        let viewport_u = Vec3::new(viewport_width, 0.0, 0.0);
        let viewport_v = Vec3::new(0.0, -viewport_height, 0.0);

        // 计算像素与像素之间的水平和垂直差值向量
        self.pixel_delta_u = viewport_u / self.image_width as f64;
        self.pixel_delta_v = viewport_v / self.image_height as f64;

        // 计算左上角像素的世界坐标
        let viewport_upper_left =
            self.center - Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
        self.pixel00_loc = viewport_upper_left + 0.5 * (self.pixel_delta_u + self.pixel_delta_v);
    }

    /// 构造从相机中心射向像素 (i, j) 的光线
    /// # 参数 `i`-像素的列索引（水平方向）`j`-像素的行索引（垂直方向）
    fn get_ray(&self, i: u32, j: u32) -> Ray {
        let offset = self.sample_square();
        let pixel_center = self.pixel00_loc
            + ((i as f64 + offset.x()) * self.pixel_delta_u)
            + ((j as f64 + offset.y()) * self.pixel_delta_v);
        let ray_direction = pixel_center - self.center;
        Ray::new(self.center, ray_direction)
    }

    // 返回一个x, y分量随机的Vec3(关于0对称)
    fn sample_square(&self) -> Vec3 {
        Vec3::new(random_double() - 0.5, random_double() - 0.5, 0.0)
    }

    /// 计算光线在场景中传播后的颜色值
    /// # 参数 `r`-入射光线 `world`-可击中物体列表的 trait 引用
    fn ray_color(&self, r: &Ray, world: &dyn Hittable) -> Color {
        let mut rec: HitRecord = HitRecord::default();

        if world.hit(r, 0.0, INFINITY, &mut rec) {
            return 0.5 * (rec.normal + Color::new(1.0, 1.0, 1.0));
        }

        let unit_direction = unit_vector(r.direction());
        let a = 0.5 * (unit_direction.y() + 1.0);
        (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
