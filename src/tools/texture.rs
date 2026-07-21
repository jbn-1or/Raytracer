#![allow(dead_code)]

use std::sync::Arc;

use crate::tools::{color::Color, interval::Interval, perlin::Perlin, vector3::Point3};

use super::rtw_image::RtwImage;

/// 纹理抽象接口，定义通过表面参数 (u, v) 和空间坐标 (p) 获取颜色的行为
pub trait Texture: Send + Sync {
    /// 计算纹理在给定表面参数 (u, v) 和空间位置 p 处的颜色
    /// # 参数
    /// * `_u` - 纹理水平坐标（沿 u 方向）
    /// * `_v` - 纹理垂直坐标（沿 v 方向）
    /// * `_p` - 击中点的世界空间坐标
    fn value(&self, _u: f64, _v: f64, _p: Point3) -> Color {
        Color::default()
    }
}

/// 纯色纹理，所有像素返回固定的反照率颜色
pub struct SolidColor {
    /// 固定的反照率颜色
    albedo: Color,
}

impl SolidColor {
    /// 使用 RGB 分量创建纯色纹理
    /// # 参数
    /// * `red`   - 红色分量（范围 [0, 1]）
    /// * `green` - 绿色分量（范围 [0, 1]）
    /// * `blue`  - 蓝色分量（范围 [0, 1]）
    pub fn new(red: f64, green: f64, blue: f64) -> Self {
        Self {
            albedo: (Color::new(red, green, blue)),
        }
    }

    /// 从颜色值创建纯色纹理
    /// # 参数
    /// * `albedo` - 反照率颜色
    pub fn from_color(albedo: Color) -> Self {
        Self { albedo }
    }
}

impl Texture for SolidColor {
    fn value(&self, _u: f64, _v: f64, _p: Point3) -> Color {
        self.albedo
    }
}

/// 棋盘格纹理，在 3D 空间中按坐标奇偶性交替显示两种子纹理
pub struct CheckerTexture {
    /// 缩放比例的倒数（1/scale），用于将世界坐标映射到棋盘格频率
    inv_scale: f64,
    /// 棋盘格子（偶）所使用的纹理
    even: Arc<dyn Texture>,
    /// 棋盘白格（奇）所使用的纹理
    odd: Arc<dyn Texture>,
}

impl CheckerTexture {
    /// 创建棋盘格纹理
    /// # 参数
    /// * `scale` - 棋盘格尺寸（世界坐标单位）
    /// * `even`  - 偶格子使用的纹理
    /// * `odd`   - 奇格子使用的纹理
    pub fn new(scale: f64, even: Arc<dyn Texture>, odd: Arc<dyn Texture>) -> Self {
        Self {
            inv_scale: 1.0 / scale,
            even,
            odd,
        }
    }

    /// 使用两种颜色创建棋盘格纹理（内部包装为 SolidColor 纹理）
    /// # 参数
    /// * `scale` - 棋盘格尺寸（世界坐标单位）
    /// * `c1`    - 格子的第一种颜色
    /// * `c2`    - 格子的第二种颜色
    pub fn from_another(scale: f64, c1: Color, c2: Color) -> Self {
        Self {
            inv_scale: 1.0 / scale,
            even: Arc::new(SolidColor::from_color(c1)),
            odd: Arc::new(SolidColor::from_color(c2)),
        }
    }
}

impl Texture for CheckerTexture {
    fn value(&self, _u: f64, _v: f64, p: Point3) -> Color {
        let x_integer = (self.inv_scale * p.x()).floor() as i32;
        let y_integer = (self.inv_scale * p.y()).floor() as i32;
        let z_integer = (self.inv_scale * p.z()).floor() as i32;

        let is_even = (x_integer + y_integer + z_integer) % 2 == 0;

        if is_even {
            self.even.value(_u, _v, p)
        } else {
            self.odd.value(_u, _v, p)
        }
    }
}

/// 图像纹理，从磁盘加载图像文件并根据 UV 坐标采样像素颜色
pub struct ImageTexture {
    /// 内部图像数据
    image: RtwImage,
}

impl ImageTexture {
    /// 从指定文件加载图像创建纹理
    /// # 参数
    /// * `filename` - 图像文件名（支持多种格式如 PNG、JPG）
    pub fn new(filename: &str) -> Self {
        Self {
            image: RtwImage::new(filename),
        }
    }
}

impl Texture for ImageTexture {
    fn value(&self, u: f64, v: f64, _p: Point3) -> Color {
        // 如果没有纹理数据，返回青色作为调试辅助
        if self.image.height() == 0 {
            return Color::new(0.0, 1.0, 1.0);
        }

        // 将纹理坐标钳制到 [0,1] x [1,0]（翻转 V 以匹配图像坐标）
        let u = Interval::new(0.0, 1.0).clamp(u);
        let v = 1.0 - Interval::new(0.0, 1.0).clamp(v); // 翻转 V

        let i = (u * self.image.width() as f64) as u32;
        let j = (v * self.image.height() as f64) as u32;
        let pixel = self.image.pixel_data(i, j);

        let color_scale = 1.0 / 255.0;
        Color::new(
            color_scale * pixel[0] as f64,
            color_scale * pixel[1] as f64,
            color_scale * pixel[2] as f64,
        )
    }
}

/// 基于 Perlin 噪声的过程纹理，用于生成自然风格的花纹
pub struct NoiseTexture {
    /// Perlin 噪声实例
    noise: Perlin,
    /// 空间缩放因子，控制噪声的频率
    scale: f64,
}

impl NoiseTexture {
    /// 创建 Perlin 噪声纹理
    /// # 参数
    /// * `scale` - 空间缩放因子（值越小，噪声变化越平缓）
    pub fn new(scale: f64) -> Self {
        Self {
            noise: Perlin::new(),
            scale,
        }
    }
}

impl Texture for NoiseTexture {
    fn value(&self, _u: f64, _v: f64, _p: Point3) -> Color {
        // Color::new(1.0, 1.0, 1.0) * self.noise.turb(&_p, 7)
        Color::new(0.5, 0.5, 0.5)
            * (1.0 + (self.scale * _p.z() + 10.0 * self.noise.turb(&_p, 7)).sin())
    }
}

/// 金色条纹纹理：将 Perlin 噪声的灰度值反向映射到金色光谱
/// 白(≈1.0) → 黑，灰(≈0.5) → 暗金，黑(≈0.0) → 亮金
pub struct GoldStripeTexture {
    /// Perlin 噪声实例
    noise: Perlin,
    /// 空间缩放因子，控制噪声的频率
    scale: f64,
}

impl GoldStripeTexture {
    /// 创建金色条纹纹理
    /// # 参数
    /// * `scale` - 空间缩放因子（值越小，噪声变化越平缓）
    pub fn new(scale: f64) -> Self {
        Self {
            noise: Perlin::new(),
            scale,
        }
    }
}

impl Texture for GoldStripeTexture {
    fn value(&self, _u: f64, _v: f64, p: Point3) -> Color {
        // 用湍流噪声 + sin 映射生成均匀分布的随机图案
        let raw = self.noise.turb(&(self.scale * p), 5);
        let n = (3.0 * raw).sin().abs();

        let bright_gold = Color::new(2.0, 1.7, 0.3);
        let dark_gold = Color::new(1.2, 0.7, 0.1);
        let black = Color::new(0.0, 0.0, 0.0);

        if n < 0.01 {
            // [0, 0.35]: 亮金 → 暗金（条纹）
            let blend = n / 0.01;
            (1.0 - blend) * bright_gold + blend * dark_gold
        } else if n < 0.03 {
            // [0.35, 0.55]: 暗金 → 黑（过渡带）
            let blend = (n - 0.03) / 0.02;
            (1.0 - blend) * dark_gold + blend * black
        } else {
            // [0.55, 1.0]: 纯黑（色块）
            black
        }
    }
}
