#![allow(dead_code)]

use console::style;
use image::RgbImage;
use indicatif::ProgressBar;

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
