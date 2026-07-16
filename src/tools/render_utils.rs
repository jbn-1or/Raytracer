#![allow(dead_code)]

use console::style;
use image::RgbImage;
use indicatif::ProgressBar;

use super::color::Color;
use image::Rgb;

/// 拼接输出路径并自动创建父目录，返回完整 PathBuf。
/// 路径相对于 CARGO_MANIFEST_DIR。
pub fn prepare_output_path(relative_path: &str) -> std::path::PathBuf {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).expect("Cannot create output directory");
    path
}

/// 根据 CI 环境变量创建合适的进度条。
/// 在 CI 中返回隐藏进度条，否则返回可见进度条。
pub fn create_progress_bar(total: u64) -> ProgressBar {
    if option_env!("CI").unwrap_or_default() == "true" {
        ProgressBar::hidden()
    } else {
        ProgressBar::new(total)
    }
}

/// 在终端打印输出路径（黄色）并将 PNG 图像保存到文件。
pub fn save_image(img: &RgbImage, path: &std::path::Path) {
    println!(
        "Output image as \"{}\"",
        style(path.to_str().unwrap()).yellow()
    );
    img.save(path).expect("Cannot save the image to the file");
}

// ============================================================
//  多线程并行渲染辅助函数
//  使用 rayon 按行并行计算像素颜色，再串行写入图片缓冲区。
// ============================================================

/// 通用并行渲染函数
///
/// 阶段 1：按行并行计算所有像素颜色（进度条实时更新）
/// 阶段 2：串行写入图片缓冲区
///
/// # 参数
/// - `img`: 可变的 RGB 图片缓冲区
/// - `width`: 图片宽度
/// - `height`: 图片高度
/// - `pixel_fn`: 接收像素坐标 `(i, j)`，返回该像素的 `Color`
/// - `write_pixel`: 接收颜色和像素引用，执行写入（含颜色转换/伽马校正等）
/// - `progress`: 进度条（在并行计算阶段实时更新）
pub fn render_parallel<F, W>(
    img: &mut RgbImage,
    width: u32,
    height: u32,
    pixel_fn: F,
    write_pixel: W,
    progress: &ProgressBar,
)
where
    F: Fn(u32, u32) -> Color + Send + Sync,
    W: Fn(Color, &mut Rgb<u8>),
{
    use rayon::prelude::*;

    // 阶段 1：按行并行计算所有像素颜色，每像素实时更新进度条
    let colors: Vec<Color> = (0..height)
        .into_par_iter()
        .flat_map(|j| {
            (0..width)
                .map(|i| {
                    let color = pixel_fn(i, j);
                    progress.inc(1);
                    color
                })
                .collect::<Vec<_>>()
        })
        .collect();

    // 阶段 2：串行写入图片（此时进度条已完成）
    for y in 0..height {
        for x in 0..width {
            let pixel = img.get_pixel_mut(x, y);
            write_pixel(colors[(y * width + x) as usize], pixel);
        }
    }
}

/// 简易并行渲染（直接写入颜色，不经过 gamma 校正）
///
/// 适用于 `write_color`（无 gamma 校正）
pub fn render_parallel_simple<F>(
    img: &mut RgbImage,
    width: u32,
    height: u32,
    pixel_fn: F,
    progress: &ProgressBar,
)
where
    F: Fn(u32, u32) -> Color + Send + Sync,
{
    render_parallel(
        img,
        width,
        height,
        pixel_fn,
        |color, pixel| Color::write_color(color, pixel),
        progress,
    );
}

/// 标准并行渲染（带 gamma 校正）
///
/// 适用于 `write_color_gamma`（带 gamma 校正）
pub fn render_parallel_gamma<F>(
    img: &mut RgbImage,
    width: u32,
    height: u32,
    pixel_fn: F,
    progress: &ProgressBar,
)
where
    F: Fn(u32, u32) -> Color + Send + Sync,
{
    render_parallel(
        img,
        width,
        height,
        pixel_fn,
        |color, pixel| Color::write_color_gamma(color, pixel),
        progress,
    );
}