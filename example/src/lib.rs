#![cfg(target_os = "android")]

use trs_24::{android_logger, overture::*};

#[no_mangle]
pub fn android_main(app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default());

    // Creates an event loop for android platforms only.
    let event_loop = EventLoopBuilder::new().with_android_app(app).build();

    // Creates an empty mutable vector named models
    let mut models = Vec::new();
    // Creates a custom duck model with specified position, scale, and rotation
    let duck = trs_24::Renderer::new_model(
        include_bytes!("../../assets/models/duck/scene.gltf"),
        include_bytes!("../../assets/models/duck/scene.bin"),
        include_bytes!("../../assets/models/duck/texture.png"),
        Vec3::new(0.5, -0.5, 0.0),
        Vec3::new(0.006, 0.006, 0.006),
        -26.0,
    );
    // Creates a custom map model with specified position, scale, and rotation
    let map = trs_24::Renderer::new_model(
        include_bytes!("../../assets/models/map/scene.gltf"),
        include_bytes!("../../assets/models/map/scene.bin"),
        include_bytes!("../../assets/models/map/texture.png"),
        Vec3::new(-0.5, 0.0, 0.0),
        Vec3::new(0.08, 0.08, 0.08),
        0.0,
    );
    // Push both duck and map models to the models vector
    models.extend([duck, map]);

    // Initialize the window, renderer, and draw the models within the models vector.
    // Second argument sets the specified RGBA color model for the world.
    App::run(event_loop, RGBA::new(0.4, 0.0, 1.0, 1.0), models);
}
