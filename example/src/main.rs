/**
 * This file is used for pre-build testing purposes.
 * Once testing phase is over, you transfer the main fn content to android_main fn in lib.rs (except the event_loop).
 * The goal of this is to speed up the testing process. You won't have to do builds every time to test something.
 */
use trs_24::overture::*;

pub fn main() {
    // WARNING: Avoid using this event_loop definition for lib.rs file.
    // Creates an event loop for non-android platforms.
    let event_loop = EventLoopBuilder::new().build();

    // Creates a custom duck model with specified position, scale, and rotation
    let duck = trs_24::renderer::Model::new(
        include_bytes!("../../assets/models/duck/scene.gltf"),
        include_bytes!("../../assets/models/duck/scene.bin"),
        include_bytes!("../../assets/models/duck/texture.png"),
        Vec3::new(0.5, -0.5, 0.0),
        Vec3::new(0.006, 0.006, 0.006),
        -26.0,
    );
    // Creates a custom map model with specified position, scale, and rotation
    let map = trs_24::renderer::Model::new(
        include_bytes!("../../assets/models/map/scene.gltf"),
        include_bytes!("../../assets/models/map/scene.bin"),
        include_bytes!("../../assets/models/map/texture.png"),
        Vec3::new(-0.5, 0.0, 0.0),
        Vec3::new(0.08, 0.08, 0.08),
        0.0,
    );

    // Initialize the window, renderer, and draw the models within the models vector.
    // Second argument sets the specified RGBA color model for the world.
    App::run(event_loop, RGBA::new(0.4, 0.0, 1.0, 1.0), vec![duck, map]);
}
