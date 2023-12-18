use crate::ui::ShapeBuilder;
use crate::ui::TextBuilder;

pub enum RotAxis {
    Roll,
    Pitch,
    Yaw,
}

#[derive(Clone, Debug)]
pub enum Shape {
    Square,
    Triangle,
}

#[derive(Clone, Debug)]
pub enum Element {
    Shape(ShapeBuilder),
    Text(TextBuilder),
}

pub struct Vec3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

impl Vec3 {
    pub fn new(x: f32, y: f32, z: f32) -> Self {
        Vec3 { x, y, z }
    }
}

#[derive(Clone, Debug)]
pub struct RGB {
    pub r: f32,
    pub g: f32,
    pub b: f32,
}

impl RGB {
    pub fn new(r: f32, g: f32, b: f32) -> Self {
        RGB { r, g, b }
    }
}

#[derive(Clone, Debug)]
pub struct RGBA {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl RGBA {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        RGBA { r, g, b, a }
    }
}
