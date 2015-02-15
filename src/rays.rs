//!A raytracer.
#![feature(path, io)]
extern crate image;

use std::old_io::File;
use std::ops::{Add, Mul, Sub};

#[derive(Copy)]
struct Vec3 {
    vals: [f32; 3]
}

impl Add<Vec3> for Vec3 {
    type Output = Vec3;
    fn add(self, other: Vec3) -> Vec3 {
        Vec3 {
            vals: [
                self.vals[0] + other.vals[0],
                self.vals[1] + other.vals[1],
                self.vals[2] + other.vals[2]
            ]
        }
    }
}

impl Sub<Vec3> for Vec3 {
    type Output = Vec3;
    fn sub(self, other: Vec3) -> Vec3 {
        Vec3 {
            vals: [
                self.vals[0] - other.vals[0],
                self.vals[1] - other.vals[1],
                self.vals[2] - other.vals[2]
            ]
        }
    }
}

impl Mul<f32> for Vec3 {
    type Output = Vec3;
    fn mul(self, other: f32) -> Vec3 {
        Vec3 {
            vals: [
                self.vals[0] * other,
                self.vals[1] * other,
                self.vals[2] * other
            ]
        }
    }
}

impl Vec3 {
    fn dot(self, other: Vec3) -> f32 {
        self.vals[0] * other.vals[0] + self.vals[1] * other.vals[1] + self.vals[2] * other.vals[2]
    }

    fn mag2(self) -> f32 {
        self.dot(self)
    }
}

#[derive(Copy)]
struct Point3 {
    inner: Vec3
}

impl Sub<Point3> for Point3 {
    type Output = Vec3;
    fn sub(self, other: Point3) -> Vec3 {
        self.inner - other.inner
    }
}

impl Add<Vec3> for Point3 {
    type Output = Point3;
    fn add(self, other: Vec3) -> Point3 {
        Point3 { inner: self.inner + other }
    }
}



#[derive(Copy)]
struct Ray3 {
    start: Point3,
    dir: Vec3
}




struct Scene {
    center: Point3,
    radius: f32
}

impl Scene {
    fn intersect(&self, ray: Ray3) -> Option<()> {
        let offset = ray.start - self.center;

        let a = ray.dir.mag2();
        let b = 2. * offset.dot(ray.dir);
        let c = offset.mag2() - self.radius*self.radius;

        let descrim = b*b - 4.*a*c;
        if descrim > 0. {
            Some(())
        } else {
            None
        }
    }
}




fn main() {
    let scene = Scene {
        center: Point3 { inner: Vec3 { vals: [0.5, 0.0, 3.0] } },
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
            start: Point3 { inner: Vec3 { vals: [0., 0., 0.] } },
            dir: Vec3 { vals: [cx, cy, 1.0] }
        };

        let value: f32 = match scene.intersect(ray) {
            Some(_) => 1.,
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
