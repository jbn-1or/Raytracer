#![allow(dead_code)]

use super::ray::Ray;
use super::rtweekend::random_double;
use super::vector3::{Point3, Vec3};

/// 相机类，负责构造并向场景分发光线。
/// Camera 的职责是纯粹的"光线工厂"——给定像素坐标，生成对应的光线。
/// 渲染循环、着色逻辑由各个场景文件自行管理。
pub struct Camera {
    /// 图像宽高比
    pub aspect_ratio: f64,
    /// 渲染图像宽度（像素）
    pub image_width: u32,
    /// 每像素发射多条光线并对颜色取平均
    pub samples_per_pixel: u32,
    /// 光线反射的最大次数
    pub max_depth: u32,

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
    /// 创建并初始化一个相机
    pub fn new(aspect_ratio: f64, image_width: u32, samples_per_pixel: u32) -> Self {
        let mut cam = Self {
            aspect_ratio,
            image_width,
            samples_per_pixel,
            max_depth: 10,
            image_height: 0,
            pixel_samples_scale: 0.0,
            center: Point3::zero(),
            pixel00_loc: Point3::zero(),
            pixel_delta_u: Vec3::zero(),
            pixel_delta_v: Vec3::zero(),
        };
        cam.initialize();
        cam
    }

    // ── getter
    pub fn image_height(&self) -> u32 {
        self.image_height
    }

    pub fn pixel_samples_scale(&self) -> f64 {
        self.pixel_samples_scale
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

    // ── 光线生成 ─────────────────────────────────────────────

    /// 构造从相机中心射向像素 (i, j) 的光线
    /// # 参数
    /// - `i`-像素的列索引（水平方向）
    /// - `j`-像素的行索引（垂直方向）
    pub fn get_ray(&self, i: u32, j: u32) -> Ray {
        let offset = self.sample_square();
        let pixel_center = self.pixel00_loc
            + ((i as f64 + offset.x()) * self.pixel_delta_u)
            + ((j as f64 + offset.y()) * self.pixel_delta_v);
        let ray_direction = pixel_center - self.center;
        Ray::new(self.center, ray_direction)
    }

    /// 返回一个 x, y 分量随机的 Vec3（关于 0 对称）
    fn sample_square(&self) -> Vec3 {
        Vec3::new(random_double() - 0.5, random_double() - 0.5, 0.0)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(1.0, 100, 10)
    }
}
