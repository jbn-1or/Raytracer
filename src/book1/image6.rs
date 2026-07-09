#![allow(dead_code)]

use crate::tools::camera::Camera;
use crate::tools::color::Color;
use crate::tools::hittable::{HitRecord, Hittable};
use crate::tools::hittable_list::HittableList;
use crate::tools::ray::Ray;
use crate::tools::rtweekend::INFINITY;
use crate::tools::sphere::Sphere;
use crate::tools::vector3::{Point3, Vec3, unit_vector};

fn ray_color(r: &Ray, world: &dyn Hittable) -> Vec3 {
    let mut rec: HitRecord = HitRecord::default();
    if world.hit(r, 0.0, INFINITY, &mut rec) {
        return 0.5 * (rec.normal + Color::new(1.0, 1.0, 1.0));
    }

    let unit_direction = unit_vector(r.direction());
    let a = 0.5 * (unit_direction.y() + 1.0);
    (1.0 - a) * Color::new(1.0, 1.0, 1.0) + a * Color::new(0.5, 0.7, 1.0)
}

pub fn render() {
    // 设定路径
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("output/book1/image6.png");

    let prefix = path.parent().unwrap();
    std::fs::create_dir_all(prefix).expect("Cannot create all the parents");

    let mut world: HittableList = HittableList::new();
    world.add(Box::new(Sphere::new(Point3::new(0.0, 0.0, -1.0), 0.5)));
    world.add(Box::new(Sphere::new(Point3::new(0.0, -100.5, -1.0), 100.0)));

    let mut cam: Camera = Camera::default();

    cam.aspect_ratio = 16.0 / 9.0;
    cam.image_width = 400;

    cam.render(&world, path);
}
