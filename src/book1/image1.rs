#![allow(dead_code)]

use console::style; // 用于在终端输出带样式的（彩色）文字
use image::{ImageBuffer, RgbImage};
// ImageBuffer: 通用图像缓冲区；RgbImage: ImageBuffer<Rgb<u8>, Vec<u8>> 的类型别名
use indicatif::ProgressBar; // 进度条库，用来显示渲染进度

pub fn render() {
    // env!("CARGO_MANIFEST_DIR") 是编译期宏，始终指向 Cargo.toml 所在的项目根目录
    // 无论从哪个目录运行程序，图片都会固定保存到 项目根目录/output/ 下
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("output/book1/image1.png");

    // 获取路径的父目录，即 "项目根目录/output/book1/"
    let prefix = path.parent().unwrap();
    // 递归创建父目录及其祖先目录（如果不存在），确保保存路径存在
    std::fs::create_dir_all(prefix).expect("Cannot create all the parents");

    let width = 256; // 图像宽度（像素）
    let height = 256; // 图像高度（像素）

    // 原文使用 .ppm 格式（现已不常用），这里改用 image crate 创建 PNG 图像
    // 你也可以换用任何你喜欢的图像格式
    let mut img: RgbImage = ImageBuffer::new(width, height);
    // 创建一个 256×256 的 RGB 图像，初始所有像素为黑色 (0, 0, 0)

    // 检查环境变量 CI 是否为 "true"（持续集成环境）
    // 如果是 CI 环境，用隐藏进度条，避免日志混乱
    // 否则显示一个总步数为 256×256 = 65536 的可见进度条
    let progress = if option_env!("CI").unwrap_or_default() == "true" {
        ProgressBar::hidden()
    } else {
        ProgressBar::new((height * width) as u64)
    };

    for j in (0..height).rev() {
        // 内层循环：从左到右扫描当前行的每个像素 (i = 0 到 255)
        for i in 0..width {
            // 获取当前坐标 (i, j) 处像素的可变引用，以便修改其颜色
            let pixel = img.get_pixel_mut(i, j);
            // 红色通道：从左到右从 0.0 线性渐变到 255.999
            // 注意使用 255.999 而非 256，是为了确保值落在 [0, 256) 区间内
            let r: f64 = (i as f64) / ((width - 1) as f64) * 255.999;

            // 绿色通道：从下到上从 255.999 线性渐变到 0.0
            // j=255 对应图像底行 → 底部绿色最大；j=0 对应图像顶行 → 顶部绿色最小
            let g: f64 = (j as f64) / ((height - 1) as f64) * 255.999;

            // 蓝色通道：固定为 0.25 × 255.999 ≈ 63.99975（暗蓝色）
            let b: f64 = 0.25 * 255.999;

            // 将计算好的 RGB 值写入像素
            *pixel = image::Rgb([r as u8, g as u8, b as u8]);
        }

        // 每完成一行渲染，进度条前进 1 步
        progress.inc(1);
    }

    // 渲染完成，结束进度条
    progress.finish();

    // 在终端打印输出路径，路径文字显示为黄色
    println!(
        "Output image as \"{}\"",
        style(path.to_str().unwrap()).yellow()
    );

    // 将图像以 PNG 格式保存到文件
    // 如果保存失败则 panic，并显示错误信息
    img.save(path).expect("Cannot save the image to the file");
}
