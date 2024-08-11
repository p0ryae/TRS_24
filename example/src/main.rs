use trs_24::{overture::*, types::*};

pub fn main() {
    // Creates an event loop for non-android platforms.
    let event_loop = EventLoop::<CustomEvent>::with_user_event().build().unwrap();
    // Create a proxy from the eventloop for custom Events
    let event_loop_proxy = event_loop.create_proxy();

    // Create a 3d duck model from its glTF model
    let duck = trs_24::renderer::Model::new(
        include_bytes!("../../assets/models/duck/scene.gltf"),
        include_bytes!("../../assets/models/duck/scene.bin"),
        include_bytes!("../../assets/models/duck/texture.png"),
    )
    .set_position(Vec3::new(0.5, -0.5, 0.0))
    .set_scale(Vec3::new(0.006, 0.006, 0.006))
    .set_rotation(-26.0, trs_24::types::RotAxis::Roll);

    // Create a 3d map model from its glTF model
    let map = trs_24::renderer::Model::new(
        include_bytes!("../../assets/models/map/scene.gltf"),
        include_bytes!("../../assets/models/map/scene.bin"),
        include_bytes!("../../assets/models/map/texture.png"),
    )
    .set_position(Vec3::new(-0.5, 0.0, 0.0))
    .set_scale(Vec3::new(0.08, 0.08, 0.08));

    // Create a shape element with square type
    let textbox = trs_24::ui::Element::new(trs_24::types::Element::Shape(
        trs_24::ui::ShapeBuilder::new(trs_24::types::Shape::Square),
    ))
    .set_color(RGBA::new(0.5, 0.0, 1.0, 0.4))
    .set_position(Vec3::new(0.0, -0.74, 0.0))
    .set_scale(Vec3::new(0.7, 0.2, 0.5));

    // Create a text element with specified font and size
    let text =
        trs_24::ui::Element::new(trs_24::types::Element::Text(trs_24::ui::TextBuilder::new(
            "TRS_24 Demo".to_string(),
            include_bytes!("../../assets/fonts/Antonio-Bold.ttf"),
            60,
        )))
        .set_color(RGBA::new(1.0, 1.0, 1.0, 1.0))
        .set_scale(Vec3::new(0.002, 0.002, 0.002))
        .set_position(Vec3::new(-0.3, -0.8, 0.0));

    // When pressing right mouse button, remove a Model and UI element with id 1
    event_loop_proxy
        .send_event(CustomEvent::Mouse(
            MouseButton::Right,
            ElementState::Pressed,
            Box::new(|scene| {
                if let Some(ref mut renderer) = scene.render_state {
                    if let Some(_) = renderer.models.get(1) {
                        renderer.models.remove(1);
                    }
                    if let Some(_) = renderer.ui.get(1) {
                        renderer.ui.remove(1);
                    }
                }
            }),
        ))
        .ok();

    // Create the scene based on eventloop
    // Run the scene with set world color, passed models, and ui elements
    Scene::new(event_loop).run(
        RGBA::new(0.1, 0.1, 0.1, 1.0),
        vec![duck, map],
        vec![textbox, text],
    );
}
