use crate::types;

pub struct Element {
    pub el_type: types::ElementType,
    pub is_hud: bool,
    pub color: types::RGBA,
    pub position: types::Vec3,
    pub scale: types::Vec3,
    pub rotation: f32,
}

impl Element {
    pub fn new(el_type: types::ElementType) -> Self {
        Self {
            el_type,
            is_hud: false,
            color: types::RGBA::new(0.1, 0.1, 0.1, 1.0),
            position: types::Vec3::new(0.0, 0.0, 0.0),
            scale: types::Vec3::new(1.0, 1.0, 1.0),
            rotation: 0.0,
        }
    }

    pub fn set_position(mut self, position: types::Vec3) -> Self {
        self.position = position;
        self
    }

    pub fn set_scale(mut self, scale: types::Vec3) -> Self {
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

    pub fn set_color(mut self, rgba: types::RGBA) -> Self {
        self.color = rgba;
        self
    }

    pub fn is_hud(mut self, is_hud: bool) -> Self {
        self.is_hud = is_hud;
        self
    }
}
