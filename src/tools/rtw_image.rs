#![allow(dead_code)]

use std::env;
use std::path::Path;

/// 图像加载辅助类，提供像素数据访问功能
pub struct RtwImage {
    /// 加载的图像宽度
    image_width: u32,
    /// 加载的图像高度
    image_height: u32,
    /// 线性8位RGB像素数据（gamma=1）
    bdata: Vec<u8>,
    /// 每像素字节数（固定为3：RGB）
    bytes_per_pixel: u32,
    /// 每扫描行字节数
    bytes_per_scanline: u32,
}

impl RtwImage {
    /// 从指定文件加载图像
    /// 如果定义了 RTW_IMAGES 环境变量，则仅在该目录中查找。
    /// 如果未找到，则依次从当前目录、images/ 子目录、父目录的 images/ 子目录
    /// 等位置搜索，最多向上查找6级目录。
    pub fn new(image_filename: &str) -> Self {
        let filename = image_filename.to_string();
        let imagedir = env::var("RTW_IMAGES").ok();

        // 在可能的路径中查找图像文件
        if let Some(ref dir) = imagedir {
            let path = format!("{}/{}", dir, image_filename);
            if let Ok(img) = Self::load(&path) {
                return img;
            }
        }
        if let Ok(img) = Self::load(&filename) {
            return img;
        }
        if let Ok(img) = Self::load(&format!("images/{}", filename)) {
            return img;
        }
        if let Ok(img) = Self::load(&format!("../images/{}", filename)) {
            return img;
        }
        if let Ok(img) = Self::load(&format!("../../images/{}", filename)) {
            return img;
        }
        if let Ok(img) = Self::load(&format!("../../../images/{}", filename)) {
            return img;
        }
        if let Ok(img) = Self::load(&format!("../../../../images/{}", filename)) {
            return img;
        }
        if let Ok(img) = Self::load(&format!("../../../../../images/{}", filename)) {
            return img;
        }
        if let Ok(img) = Self::load(&format!("../../../../../../images/{}", filename)) {
            return img;
        }

        eprintln!("ERROR: Could not load image file '{}'.", image_filename);

        // 返回空图像（width=0, height=0, bdata=empty）
        Self {
            image_width: 0,
            image_height: 0,
            bdata: Vec::new(),
            bytes_per_pixel: 3,
            bytes_per_scanline: 0,
        }
    }

    /// 加载指定路径的图像文件，返回转换后的字节数据
    /// 加载线性（gamma=1）图像数据
    fn load(filename: &str) -> Result<Self, image::ImageError> {
        let path = Path::new(filename);
        if !path.exists() {
            // 文件不存在时返回一个可忽略的错误
            return Err(image::ImageError::Limits(
                image::error::LimitError::from_kind(
                    image::error::LimitErrorKind::InsufficientMemory,
                ),
            ));
        }

        let img = image::open(path)?;
        let img = img.to_rgb8(); // 转换为 RGB8 格式

        let image_width = img.width();
        let image_height = img.height();
        let bytes_per_pixel = 3u32;
        let bytes_per_scanline = image_width * bytes_per_pixel;

        // 获取原始字节数据（sRGB 编码的字节）
        let raw_data = img.into_raw();

        // 将 sRGB 编码的字节转换为线性空间字节
        // sRGB → 线性：linear = (srgb / 255.0)^2.2 * 255.0
        let bdata: Vec<u8> = raw_data
            .iter()
            .map(|&b| {
                let linear = (b as f64 / 255.0).powf(2.2);
                (linear * 255.0).round().clamp(0.0, 255.0) as u8
            })
            .collect();

        Ok(Self {
            image_width,
            image_height,
            bdata,
            bytes_per_pixel,
            bytes_per_scanline,
        })
    }

    /// 返回图像宽度，若无数据则返回 0
    pub fn width(&self) -> u32 {
        self.image_width
    }

    /// 返回图像高度，若无数据则返回 0
    pub fn height(&self) -> u32 {
        self.image_height
    }

    /// 返回像素 (x, y) 的三个 RGB 字节值的引用。
    /// 如果没有图像数据，返回品红色 [255, 0, 255]。
    pub fn pixel_data(&self, x: u32, y: u32) -> &[u8] {
        // 无图像数据时，返回品红色
        if self.bdata.is_empty() {
            return &[255, 0, 255];
        }

        let x = clamp(x, 0, self.image_width);
        let y = clamp(y, 0, self.image_height);

        let index = (y * self.bytes_per_scanline + x * self.bytes_per_pixel) as usize;
        &self.bdata[index..index + self.bytes_per_pixel as usize]
    }
}

/// 将值钳制到范围 [low, high)
fn clamp(x: u32, low: u32, high: u32) -> u32 {
    if x < low {
        return low;
    }
    if x < high {
        return x;
    }
    high - 1
}
