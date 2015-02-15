//!A raytracer.
#![feature(path, io)]
extern crate image;

use std::old_io::File;
use std::num::Float;

use point::Point3;
use vec::Vec3;

mod vec;
mod point;

#[derive(Copy)]
struct Ray3 {
    start: Point3,
    dir: Vec3
}

struct Scene {
    center: Point3,
    radius: f32
}

struct Intersection {
    time: f32,
    normal: Vec3
}

impl Scene {
    fn intersect(&self, ray: Ray3) -> Option<Intersection> {
        let offset = ray.start - self.center;

        let a = ray.dir.mag2();
        let b = 2. * offset.dot(ray.dir);
        let c = offset.mag2() - self.radius*self.radius;

        let descrim = b*b - 4.*a*c;
        if descrim > 0. {
            let t1 = (-b - descrim.sqrt()) / (2. * a);
            let t2 = (-b + descrim.sqrt()) / (2. * a);
            if t1 > 0. {
                let normal = ray.start + ray.dir * t1 - self.center;
                Some(Intersection { time: t1, normal: normal })
            } else if t2 > 0. {
                let normal = ray.start + ray.dir * t2 - self.center;
                Some(Intersection { time: t2, normal: normal })
            } else {
                None
            }
        } else {
            None
        }
    }
}




fn main() {
    let scene = Scene {
        center: Point3::new(1.5, 0.0, 5.0),
        radius: 0.3
    };

    let imgx = 800;
    let imgy = 800;

    let scalex = 4.0 / imgx as f32;
    let scaley = 4.0 / imgy as f32;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    // Iterate over the coordiantes and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let cy = y as f32 * scaley - 2.0;
        let cx = x as f32 * scalex - 2.0;

        let ray = Ray3 {
            start: Point3::new(0., 0., -8.0),
            dir: Vec3::new(cx, cy, 8.0)
        };

        let value: f32 = match scene.intersect(ray) {
            Some(intersection) => {
                let scale = -(intersection.normal.mag2() * ray.dir.mag2()).sqrt();
                intersection.normal.dot(ray.dir) / scale
            },
            None => 0.
        };

        // Create an 8bit pixel of type Luma and value i
        // and assign in to the pixel at position (x, y)
        *pixel = image::Luma([(value*255.) as u8]);
    }

    // Save the image as “out.png”
    let ref mut fout = File::create(&Path::new("out.png")).unwrap();

    // We must indicate the image’s color type and what format to save as
    let _ = image::ImageLuma8(imgbuf).save(fout, image::PNG);
}
