//!A raytracer.
#![feature(path, io)]
extern crate image;
extern crate rand;

use std::old_io::File;
use std::num::Float;
use std::f32::consts::PI_2;

use rand::distributions::IndependentSample;

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
    object: ObjectId
}

trait Scene {
    // TODO: Should this have a "not present" id?
    type ObjectId: Copy;

    fn intersect(&self, ray: Ray3, previous: Option<Self::ObjectId>) -> Option<Intersection<Self::ObjectId>>;
}

struct Sphere {
    center: Point3,
    radius: f32
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
                    object: ()
                })
            } else if t2 > 0. {
                let p = ray.start + ray.dir * t2;
                let normal = p - self.center;
                Some(Intersection {
                    time: t2,
                    point: p,
                    normal: normal,
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
    normal: Vec3
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
                            object: Choice::OptA(ai.object)
                        })
                    } else {
                        Some(Intersection {
                            time: bi.time,
                            normal: bi.normal,
                            point: bi.point,
                            object: Choice::OptB(bi.object)
                        })
                    },
                    None => Some(Intersection {
                        time: ai.time,
                        normal: ai.normal,
                        point: ai.point,
                        object: Choice::OptA(ai.object)
                    })
                }
            },
            None => match self.1.intersect(ray, bid) {
                Some(bi) => Some(Intersection {
                    time: bi.time,
                    normal: bi.normal,
                    point: bi.point,
                    object: Choice::OptB(bi.object)
                }),
                None => None
            }
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

fn rand_sphere<R: rand::Rng>(rng: &mut R) -> Vec3 {
    let z = rand::distributions::Range::new(-1., 1.).ind_sample(rng);
    let r = (1. - z*z).sqrt();
    let angle = rand::distributions::Range::new(0., PI_2).ind_sample(rng);
    Vec3::new(r*angle.cos(), r*angle.sin(), z)
}

fn main() {
    let mut rng = rand::thread_rng();

    let num_bounces = 10;
    let rays_per_pixel = 50;
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

        let mut total_light = 0.;
        for _ in 0..rays_per_pixel {
            let mut ray = Ray3 {
                start: Point3::new(0., 0., -8.0),
                dir: Vec3::new(cx, cy, 8.0)
            };
            let mut prev_object = None;
            let mut strength = 1.;
            for _ in 0..num_bounces {
                if let Some(intersection) = scene.intersect(ray, prev_object) {
                    prev_object = Some(intersection.object);

                    let light_ray = Ray3 {
                        start: intersection.point,
                        dir: light - intersection.point
                    };

                    let can_see_light = match scene.intersect(light_ray, prev_object) {
                        Some(light_intersection) => {
                            light_intersection.time > 1.
                        },
                        None => true
                    };

                    if can_see_light {
                        let light_strength = 10.0 / light_ray.dir.mag2();
                        let scale = (intersection.normal.mag2() * light_ray.dir.mag2()).sqrt();
                        total_light += strength * light_strength * intersection.normal.dot(light_ray.dir) / scale;
                    }

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
                    strength *= dir.dot(intersection.normal) / intersection.normal.mag2().sqrt();
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
