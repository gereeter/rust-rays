//!A raytracer.
#![feature(core, os, old_path, old_io)]
extern crate image;
extern crate rand;

use std::old_io::File;
use std::thread;
use std::os;

use rand::Rng;

use point::Point3;
use vec::Vec3;
use ray::Ray3;
use distribution::Distribution;
use material::{Material, Diffuse, Specular};
use scene::Scene;

use sphere::Sphere;
use plane::Plane;

mod vec;
mod point;
mod ray;
mod distribution;

mod material;
mod scene;

mod sphere;
mod plane;

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
    let rays_per_pixel = 200;
    let scene: ((&[_],_),&[_]) = ((
        &[Sphere::new(
            Point3::new(1.2, 0.0, 5.0),
            0.3,
            Material::new(Diffuse, 0.)
        ),
        Sphere::new(
            Point3::new(1.5, 0.0, 3.5),
            0.35,
            Material::new(Diffuse, 0.)
        ),
        Sphere::new(
            Point3::new(1.0, 2.5, 0.5),
            1.0,
            Material::new(Diffuse, 30.)
        )],
        &Sphere::new(
            Point3::new(-1.5, 0.0, 3.5),
            1.0,
            Material::new(Specular, 0.)
        )),
        &[Plane::new(
            Point3::new(0.0, 0.0, 8.0),
            Vec3::new(2.0, 0.0, -1.0),
            Material::new(Diffuse, 0.)
        ),
        Plane::new(
            Point3::new(0.0, 0.0, 8.0),
            Vec3::new(-1.0, 0.0, -2.0),
            Material::new(Diffuse, 0.)
        ),
        Plane::new(
            Point3::new(0.0, 2.0, 0.0),
            Vec3::new(0.0, -1.0, 0.0),
            Material::new(Diffuse, 0.)
        ),
        Plane::new(
            Point3::new(0.0, -3.0, 0.0),
            Vec3::new(0.0, 1.0, 0.0),
            Material::new(Diffuse, 0.)
        ),
        Plane::new(
            Point3::new(0.0, 0.0, -10.0),
            Vec3::new(0.0, 0.0, 1.0),
            Material::new(Diffuse, 0.)
        )]
    );

    let num_threads = os::num_cpus();

    let imgx = 800;
    let imgy = 800;

    let scalex = 4.0 / imgx as f32;
    let scaley = 4.0 / imgy as f32;

    // Create a new ImgBuf with width: imgx and height: imgy
    let mut imgbuf = image::ImageBuffer::new(imgx, imgy);

    // Iterate over the pixels in parallel
    {
        let mut img_data = imgbuf.as_mut_slice();
        let chunk_size = img_data.len() / num_threads + 1;
        img_data.chunks_mut(chunk_size).enumerate().map(|(chunk_i, chunk)| {
            thread::scoped(move || {
                let mut rng = rand::weak_rng();

                let base = (chunk_i * chunk_size) as u32;
                let mut x = base % imgx;
                let mut y = base / imgx;
                for pixel in chunk.iter_mut() {
                    x += 1;
                    if x == imgx {
                        x = 0;
                        y += 1;
                    }

                    let cy = -(y as f32 * scaley - 2.0);
                    let cx = x as f32 * scalex - 2.0;

                    let mut total_light = 0.;
                    for _ in 0..rays_per_pixel {
                        let mut ray = Ray3 {
                            start: Point3::new(0., 0., -8.0),
                            dir: Vec3::new(cx, cy, 8.0)
                        };
                        let mut prev_object = None;
                        let mut scale = 1.;
                        loop {
                            if let Some(intersection) = scene.intersect(ray, prev_object) {
                                total_light += scale * intersection.emitted;

                                let (reflected, new_ray, object) = intersection.reflection.sample(&mut rng);
                                ray = new_ray;
                                prev_object = Some(object);

                                if scale > 0.2 {
                                    scale *= reflected;
                                } else if rng.next_f32() > reflected {
                                    break;
                                }
                            } else {
                                break;
                            }
                        }
                    }

                    // Create an 8bit pixel of type Luma and value i
                    // and assign in to the pixel at position (x, y)
                    *pixel = (clamp(total_light / rays_per_pixel as f32)*255.) as u8;
                }
            })
        }).collect::<Vec<_>>();
    }

    // Save the image as “out.png”
    let ref mut fout = File::create(&Path::new("out.png")).unwrap();

    // We must indicate the image’s color type and what format to save as
    let _ = image::ImageLuma8(imgbuf).save(fout, image::PNG);
}
