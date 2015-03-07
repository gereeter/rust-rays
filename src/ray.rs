use point::Point3;
use vec::Vec3;

#[derive(Copy, Clone)]
pub struct Ray3 {
    pub start: Point3,
    pub dir: Vec3
}
