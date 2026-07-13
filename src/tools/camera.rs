#![allow(dead_code)]

use crate::tools::rtweekend::degrees_to_radians;
use crate::tools::vector3::{cross, unit_vector};

use super::ray::Ray;
use super::rtweekend::random_double;
use super::vector3::{Point3, Vec3, random_in_unit_disk};

/// 相机类，负责构造并向场景分发光线。
/// Camera 的职责是纯粹的"光线工厂"——给定像素坐标，生成对应的光线。
/// 渲染循环、着色逻辑由各个场景文件自行管理。
pub struct Camera {
    // ------------公有成员
    /// 图像宽高比
    pub aspect_ratio: f64,
    /// 渲染图像宽度（像素）
    pub image_width: u32,
    /// 每像素发射多条光线并对颜色取平均
    pub samples_per_pixel: u32,
    /// 光线反射的最大次数
    pub max_depth: u32,

    /// 垂直视场角（fov）
    pub vfov: f64,
    /// 相机look from的点
    pub lookfrom: Point3,
    /// 相机look at的点
    pub lookat: Point3,
    /// 相机平面上的“相对向上”方向
    pub vup: Vec3,
    /// 每个像素所对的光锥角
    pub defocus_angle: f64,
    /// 相机lookfrom点到像平面的距离
    pub focus_dist: f64,

    // -----------私有成员
    /// 渲染图像高度
    image_height: u32,
    /// 像素采样缩放因子（= 1.0 / samples_per_pixel）
    pixel_samples_scale: f64,
    /// 相机中心位置
    center: Point3,
    /// 像素 (0, 0) 的世界坐标
    pixel00_loc: Point3,
    /// 水平方向相邻像素的偏移向量
    pixel_delta_u: Vec3,
    /// 垂直方向相邻像素的偏移向量
    pixel_delta_v: Vec3,
    /// 散焦圆盘水平半径
    defocus_dist_u: Vec3,
    /// 散焦圆盘竖直方向半径
    defocus_dist_v: Vec3,

    // 相机坐标系向量
    u: Vec3,
    v: Vec3,
    w: Vec3,
}

impl Camera {
    /// 创建并初始化一个相机（使用默认参数）
    pub fn new() -> Self {
        Self {
            aspect_ratio: 1.0,
            image_width: 100,
            samples_per_pixel: 10,
            max_depth: 10,
            vfov: 90.0,
            lookfrom: Point3::zero(),
            lookat: Point3::new(0.0, 0.0, -1.0),
            vup: Vec3::new(0.0, 1.0, 0.0),
            defocus_angle: 0.0,
            focus_dist: 10.0,

            image_height: 0,
            pixel_samples_scale: 0.0,
            center: Point3::zero(),
            pixel00_loc: Point3::zero(),
            pixel_delta_u: Vec3::zero(),
            pixel_delta_v: Vec3::zero(),
            u: Vec3::zero(),
            v: Vec3::zero(),
            w: Vec3::zero(),
            defocus_dist_u: Vec3::zero(),
            defocus_dist_v: Vec3::zero(),
        }
    }

    // ── getter
    /// 获取渲染图像高度
    pub fn image_height(&self) -> u32 {
        self.image_height
    }

    /// 获取像素采样缩放因子
    pub fn pixel_samples_scale(&self) -> f64 {
        self.pixel_samples_scale
    }

    /// 初始化相机参数：计算图像高度、视口尺寸、像素偏移等
    /// 必须在设置完 `aspect_ratio`、`image_width`、`samples_per_pixel`、
    /// `max_depth`、`vfov`、`lookfrom`、`lookat`、`vup` 等参数后手动调用。
    pub fn initialize(&mut self) {
        self.image_height = (self.image_width as f64 / self.aspect_ratio) as u32;
        if self.image_height < 1 {
            self.image_height = 1;
        }

        self.pixel_samples_scale = 1.0 / self.samples_per_pixel as f64;

        self.center = self.lookfrom;

        // 向后兼容：若 focus_dist 未显式设置（仍为默认值 10.0），
        // 则用 lookfrom 到 lookat 的距离作为实际焦距
        let focal_length = (self.lookfrom - self.lookat).length();
        let dist_to_focus = if self.focus_dist == 10.0 {
            focal_length
        } else {
            self.focus_dist
        };

        // 确定视口尺寸（基于 focus_dist）
        let theta = degrees_to_radians(self.vfov);
        let h = f64::tan(theta / 2.0);
        let viewport_height = 2.0 * h * dist_to_focus;
        let viewport_width = viewport_height * (self.image_width as f64 / self.image_height as f64);

        self.w = unit_vector(self.lookfrom - self.lookat);
        self.u = unit_vector(cross(self.vup, self.w));
        self.v = cross(self.w, self.u);

        // 计算水平与垂直方向跨越视口边缘的向量
        let viewport_u = viewport_width * self.u;
        let viewport_v = -viewport_height * self.v;

        // 计算像素与像素之间的水平和垂直差值向量
        self.pixel_delta_u = viewport_u / self.image_width as f64;
        self.pixel_delta_v = viewport_v / self.image_height as f64;

        // 计算左上角像素的世界坐标
        let viewport_upper_left =
            self.center - dist_to_focus * self.w - viewport_u / 2.0 - viewport_v / 2.0;
        self.pixel00_loc = viewport_upper_left + 0.5 * (self.pixel_delta_u + self.pixel_delta_v);

        // 计算散焦圆盘基向量
        let defocus_radius = dist_to_focus * f64::tan(degrees_to_radians(self.defocus_angle / 2.0));
        self.defocus_dist_u = self.u * defocus_radius;
        self.defocus_dist_v = self.v * defocus_radius;
    }

    // ── 光线生成 ─────────────────────────────────────────────

    /// 构造从相机中心（或散焦圆盘）射向像素 (i, j) 的光线
    /// # 参数
    /// - `i`-像素的列索引（水平方向）
    /// - `j`-像素的行索引（垂直方向）
    ///
    /// 当 `defocus_angle <= 0` 时，光线从 `center` 发出（兼容旧行为）；
    /// 当 `defocus_angle > 0` 时，从散焦圆盘上的随机点发出光线。
    pub fn get_ray(&self, i: u32, j: u32) -> Ray {
        let offset = self.sample_square();
        let pixel_center = self.pixel00_loc
            + ((i as f64 + offset.x()) * self.pixel_delta_u)
            + ((j as f64 + offset.y()) * self.pixel_delta_v);

        let ray_origin = if self.defocus_angle <= 0.0 {
            self.center
        } else {
            self.defocus_disk_sample()
        };
        let ray_direction = pixel_center - ray_origin;
        let ray_time = random_double();
        Ray::new_with_time(ray_origin, ray_direction, ray_time)
    }

    /// 返回 x, y 分量在 [-0.5, 0.5) 范围内随机的 Vec3，z 分量为 0
    fn sample_square(&self) -> Vec3 {
        Vec3::new(random_double() - 0.5, random_double() - 0.5, 0.0)
    }

    /// 返回散焦圆盘上的一个随机采样点
    fn defocus_disk_sample(&self) -> Point3 {
        let p = random_in_unit_disk();
        self.center + (p[0] * self.defocus_dist_u) + (p[1] * self.defocus_dist_v)
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new()
    }
}
