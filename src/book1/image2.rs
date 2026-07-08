use crate::tools::color::Color;
use crate::tools::ray::Ray;
use crate::tools::vector3::{Point3, Vec3, unit_vector};
use console::style;
use image::{ImageBuffer, RgbImage};
use indicatif::ProgressBar;

fn ray_color(r: &Color) -> Vec3 {
    let unit_direction = unit_vector(*r);
    let a = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
}

pub fn render() {
    // 设定路径
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("output/book1/image2.png");

    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).expect("Cannot create all the parents");

    // 设定照片视口尺寸
    let aspect_ratio = 16.0 / 9.0;
    let image_width = 400;

    let mut image_height: u32 = ((image_width) as f64 / aspect_ratio) as u32;
    if image_height < 1 {
        image_height = 1;
    }
    let focal_length = 1.0;
    let viewport_height = 2.0;
    let viewport_width: f64 = viewport_height * (image_width / image_height) as f64;
    let camera_center = Point3::new(0.0, 0.0, 0.0);

    // 计算视口水平方向与垂直向下方向边缘的向量。
    let viewport_u: Vec3 = Vec3::new(viewport_width, 0.0, 0.0);
    let viewport_v: Vec3 = Vec3::new(0.0, -viewport_height, 0.0);

    // 计算像素与像素之间的水平和垂直差值向量。
    let pixel_delta_u = viewport_u / (image_width) as f64;
    let pixel_delta_v = viewport_v / (image_height) as f64;

    // 计算左上角像素的位置。
    let viewport_upper_left =
        camera_center - Vec3::new(0.0, 0.0, focal_length) - viewport_u / 2.0 - viewport_v / 2.0;
    let pixel00_loc = viewport_upper_left + 0.5 * (pixel_delta_u + pixel_delta_v);

    // 产生新图片
    let mut img: RgbImage = ImageBuffer::new(image_width, image_height);

    let progress = if option_env!("CI").unwrap_or_default() == "true" {
        ProgressBar::hidden()
    } else {
        ProgressBar::new((image_height * image_width) as u64)
    };

    // 渲染图片
    for j in 1..image_height {
        for i in 1..image_width {
            let pixel_center =
                pixel00_loc + (i as f64 * pixel_delta_u) + (j as f64 * pixel_delta_v);
            let ray_direction = pixel_center - camera_center;
            let r = Ray::new(&camera_center, &ray_direction);

            let pixel_color = ray_color(&r.direction());
            let pixel = img.get_pixel_mut(i, j);
            Color::write_color(&pixel_color, pixel);
        }
    }

    progress.finish();

    println!(
        "Output image as \"{}\"",
        style(path.to_str().unwrap()).yellow()
    );
    img.save(path).expect("Cannot save the image to the file");
}
