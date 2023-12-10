use crate::overture::*;
use crate::types::*;

pub struct Element {
    pub el_type: ElementType,
    pub is_hud: bool,
    pub color: RGBA,
    pub position: Vec3,
    pub scale: Vec3,
    pub rotation: f32
}

impl Element {
    pub fn new(
        el_type: ElementType
    ) -> Self {
        Self {
            el_type,
            is_hud: false,
            color: RGBA::new(0.1, 0.1, 0.1, 1.0),
            position: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(1.0, 1.0, 1.0),
            rotation: 0.0
        }
    }

    pub fn set_position(mut self, position: Vec3) -> Self {
        self.position = position;
        self
    }

    pub fn set_scale(mut self, scale: Vec3) -> Self {
        self.scale = scale;
        self
    }

    pub fn set_rotation(mut self, rotation: f32) -> Self {
        self.rotation = rotation;
        self
    }

    // pub fn set_image(mut self, image: &'static [u8]) -> Self {
    //    self.image = Some(image);
    //    self
    //}

    pub fn set_color(mut self, rgba: RGBA) -> Self {
        self.color = rgba;
        self
    }

    pub fn is_hud(mut self, is_hud: bool) -> Self {
        self.is_hud = is_hud;
        self
    }
}