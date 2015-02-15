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

struct Intersection {
    time: f32,
    point: Point3,
    normal: Vec3
}

trait Scene {
    fn intersect(&self, ray: Ray3) -> Option<Intersection>;
}

struct Sphere {
    center: Point3,
    radius: f32
}

impl Scene for Sphere {
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
                let p = ray.start + ray.dir * t1;
                let normal = p - self.center;
                Some(Intersection {
                    time: t1,
                    point: p,
                    normal: normal
                })
            } else if t2 > 0. {
                let p = ray.start + ray.dir * t2;
                let normal = p - self.center;
                Some(Intersection {
                    time: t2,
                    point: p,
                    normal: normal
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}

struct Plane {
    origin: Point3,
    normal: Vec3
}

impl Scene for Plane {
    fn intersect(&self, ray: Ray3) -> Option<Intersection> {
        let divisor = ray.dir.dot(self.normal);
        if divisor == 0. {
            None
        } else {
            let offset = ray.start - self.origin;
            let time = -offset.dot(self.normal) / divisor;
            if time > 0. {
                Some(Intersection {
                    time: time,
                    point: ray.start + ray.dir * time,
                    normal: self.normal
                })
            } else {
                None
            }
        }
    }
}

impl<A: Scene, B: Scene> Scene for (A, B) {
    fn intersect(&self, ray: Ray3) -> Option<Intersection> {
        match self.0.intersect(ray) {
            Some(ai) => {
                match self.1.intersect(ray) {
                    Some(bi) => if ai.time < bi.time {
                        Some(ai)
                    } else {
                        Some(bi)
                    },
                    None => Some(ai)
                }
            },
            None => self.1.intersect(ray)
        }
    }
}

fn clamp(val: f32) -> f32 {
    if val > 1. {
        1.
    } else if val < 0. {
        0.
    } else {
        val
    }
}

fn main() {
    let scene = (
        (Sphere {
            center: Point3::new(1.2, 0.0, 5.0),
            radius: 0.3
        },
        Sphere {
            center: Point3::new(1.5, 0.0, 3.5),
            radius: 0.35
        }),
        (Plane {
            origin: Point3::new(0.0, 0.0, 8.0),
            normal: Vec3::new(2.0, 0.0, -1.0)
        },
        Plane {
            origin: Point3::new(0.0, 0.0, 8.0),
            normal: Vec3::new(-1.0, 0.0, -2.0)
        })
    );
    let light = Point3::new(1.0, -1.5, 0.5);

    let imgx = 800;
    let imgy = 800;

    let scalex = 4.0 / imgx as f32;
    let scaley = 4.0 / imgy as f32;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    // Iterate over the coordiantes and pixels of the image
    for (x, y, pixel) in imgbuf.enumerate_pixels_mut() {
        let cy = -(y as f32 * scaley - 2.0);
        let cx = x as f32 * scalex - 2.0;

        let ray = Ray3 {
            start: Point3::new(0., 0., -8.0),
            dir: Vec3::new(cx, cy, 8.0)
        };

        let value: f32 = match scene.intersect(ray) {
            Some(intersection) => {
                let light_dir = light - intersection.point;
                let light_strength = 10.0 / light_dir.mag2();
                let scale = (intersection.normal.mag2() * light_dir.mag2()).sqrt();
                clamp(light_strength * intersection.normal.dot(light_dir) / scale)
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
