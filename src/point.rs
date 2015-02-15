use std::ops::{Add, Sub};
use vec::Vec3;

#[derive(Copy)]
pub struct Point3 {
    inner: Vec3
}

impl Point3 {
	pub fn new(x: f32, y: f32, z: f32) -> Point3 {
		Point3 {
			inner: Vec3::new(x, y, z)
		}
	}
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
