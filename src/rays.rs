//!A raytracer.
#![feature(path, io)]
extern crate image;
extern crate rand;

use std::old_io::File;
use std::num::Float;
use std::f32::consts::PI_2;

use rand::distributions::IndependentSample;
use rand::Rng;

use point::Point3;
use vec::Vec3;

mod vec;
mod point;

#[derive(Copy)]
struct Ray3 {
    start: Point3,
    dir: Vec3
}

struct Intersection<ObjectId> {
    time: f32,
    point: Point3,
    normal: Vec3,
    emitted: f32,
    object: ObjectId
}

trait Scene {
    // TODO: Should this have a "not present" id?
    type ObjectId: Copy;

    fn intersect(&self, ray: Ray3, previous: Option<Self::ObjectId>) -> Option<Intersection<Self::ObjectId>>;
}

struct Material {
    emitted_radiance: f32
}

struct Sphere {
    center: Point3,
    radius: f32,
    material: Material
}

impl Scene for Sphere {
    type ObjectId = ();
    fn intersect(&self, ray: Ray3, previous: Option<()>) -> Option<Intersection<()>> {
        if let Some(()) = previous {
            return None;
        }

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
                    normal: normal,
                    emitted: self.material.emitted_radiance,
                    object: ()
                })
            } else if t2 > 0. {
                let p = ray.start + ray.dir * t2;
                let normal = p - self.center;
                Some(Intersection {
                    time: t2,
                    point: p,
                    normal: normal,
                    emitted: self.material.emitted_radiance,
                    object: ()
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
    normal: Vec3,
    material: Material
}

impl Scene for Plane {
    type ObjectId = ();
    fn intersect(&self, ray: Ray3, previous: Option<()>) -> Option<Intersection<()>> {
        if let Some(()) = previous {
            return None;
        }

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
                    normal: self.normal,
                    emitted: self.material.emitted_radiance,
                    object: ()
                })
            } else {
                None
            }
        }
    }
}

#[derive(Copy)]
enum Choice<A, B> {
    OptA(A),
    OptB(B)
}

impl<A: Scene, B: Scene> Scene for (A, B) {
    type ObjectId = Choice<A::ObjectId, B::ObjectId>;
    fn intersect(&self, ray: Ray3, previous: Option<Choice<A::ObjectId, B::ObjectId>>) -> Option<Intersection<Choice<A::ObjectId, B::ObjectId>>> {
        let (aid, bid) = match previous {
            None => (None, None),
            Some(Choice::OptA(aid)) => (Some(aid), None),
            Some(Choice::OptB(bid)) => (None, Some(bid))
        };

        match self.0.intersect(ray, aid) {
            Some(ai) => {
                match self.1.intersect(ray, bid) {
                    Some(bi) => if ai.time < bi.time {
                        Some(Intersection {
                            time: ai.time,
                            normal: ai.normal,
                            point: ai.point,
                            emitted: ai.emitted,
                            object: Choice::OptA(ai.object)
                        })
                    } else {
                        Some(Intersection {
                            time: bi.time,
                            normal: bi.normal,
                            point: bi.point,
                            emitted: bi.emitted,
                            object: Choice::OptB(bi.object)
                        })
                    },
                    None => Some(Intersection {
                        time: ai.time,
                        normal: ai.normal,
                        point: ai.point,
                        emitted: ai.emitted,
                        object: Choice::OptA(ai.object)
                    })
                }
            },
            None => match self.1.intersect(ray, bid) {
                Some(bi) => Some(Intersection {
                    time: bi.time,
                    normal: bi.normal,
                    point: bi.point,
                    emitted: bi.emitted,
                    object: Choice::OptB(bi.object)
                }),
                None => None
            }
        }
    }
}

impl<T: Scene> Scene for [T] {
    type ObjectId = (usize, T::ObjectId);
    fn intersect(&self, ray: Ray3, previous: Option<(usize, T::ObjectId)>) -> Option<Intersection<(usize, T::ObjectId)>> {
        let mut best: Option<Intersection<(usize, T::ObjectId)>> = None;

        for (id, obj) in self.iter().enumerate() {
            let prev = match previous {
                Some((prev_index, prev_obj)) => if prev_index == id {
                    Some(prev_obj)
                } else {
                    None
                },
                None => None
            };

            if let Some(intersection) = obj.intersect(ray, prev) {
                let new_best = match best {
                    Some(ref cur_best) => intersection.time < cur_best.time,
                    None => true
                };

                if new_best {
                    best = Some(Intersection {
                        time: intersection.time,
                        normal: intersection.normal,
                        point: intersection.point,
                        emitted: intersection.emitted,
                        object: (id, intersection.object)
                    });
                }
            }
        }

        best
    }
}

impl<'a, T: ?Sized + Scene> Scene for &'a T {
    type ObjectId = T::ObjectId;
    fn intersect(&self, ray: Ray3, previous: Option<T::ObjectId>) -> Option<Intersection<T::ObjectId>> {
        (*self).intersect(ray, previous)
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

fn rand_sphere<R: rand::Rng>(rng: &mut R) -> Vec3 {
    let z = rand::distributions::Range::new(-1., 1.).ind_sample(rng);
    let r = (1. - z*z).sqrt();
    let angle = rand::distributions::Range::new(0., PI_2).ind_sample(rng);
    Vec3::new(r*angle.cos(), r*angle.sin(), z)
}

fn main() {
    let mut rng = rand::thread_rng();

    let rays_per_pixel = 100;
    let scene: (&[Sphere], &[Plane]) = (
        &[Sphere {
            center: Point3::new(1.2, 0.0, 5.0),
            radius: 0.3,
            material: Material { emitted_radiance: 0. }
        },
        Sphere {
            center: Point3::new(1.5, 0.0, 3.5),
            radius: 0.35,
            material: Material { emitted_radiance: 0. }
        },
        Sphere {
            center: Point3::new(1.0, 2.5, 0.5),
            radius: 1.0,
            material: Material { emitted_radiance: 20. }
        }],
        &[Plane {
            origin: Point3::new(0.0, 0.0, 8.0),
            normal: Vec3::new(2.0, 0.0, -1.0),
            material: Material { emitted_radiance: 0. }
        },
        Plane {
            origin: Point3::new(0.0, 0.0, 8.0),
            normal: Vec3::new(-1.0, 0.0, -2.0),
            material: Material { emitted_radiance: 0. }
        },
        Plane {
            origin: Point3::new(0.0, 2.0, 0.0),
            normal: Vec3::new(0.0, -1.0, 0.0),
            material: Material { emitted_radiance: 0. }
        },
        Plane {
            origin: Point3::new(0.0, -3.0, 0.0),
            normal: Vec3::new(0.0, 1.0, 0.0),
            material: Material { emitted_radiance: 0. }
        }]
    );

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
                    prev_object = Some(intersection.object);

                    total_light += scale * intersection.emitted;

                    let cand_dir = rand_sphere(&mut rng);
                    let dir = if cand_dir.dot(intersection.normal) < 0. {
                        -cand_dir
                    } else {
                        cand_dir
                    };
                    ray = Ray3 {
                        start: intersection.point,
                        dir: dir
                    };

                    let p = dir.dot(intersection.normal) / intersection.normal.mag2().sqrt();
                    if scale > 0.2 {
                        scale *= p;
                    } else if rng.next_f32() > p {
                        break;
                    }
                } else {
                    break;
                }
            }
        }

        // Create an 8bit pixel of type Luma and value i
        // and assign in to the pixel at position (x, y)
        *pixel = image::Luma([(clamp(total_light / rays_per_pixel as f32)*255.) as u8]);
    }

    // Save the image as “out.png”
    let ref mut fout = File::create(&Path::new("out.png")).unwrap();

    // We must indicate the image’s color type and what format to save as
    let _ = image::ImageLuma8(imgbuf).save(fout, image::PNG);
}
