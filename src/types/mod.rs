use crate::ui::ShapeBuilder;
use crate::ui::TextBuilder;

pub enum RotAxis {
    Roll,
    Pitch,
    Yaw,
}

#[derive(Clone)]
pub enum Shape {
    Square,
    Triangle,
}

pub enum ElementType {
    Shape(ShapeBuilder),
    Text(TextBuilder),
}
