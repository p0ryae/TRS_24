#![allow(dead_code)]

pub mod renderer;
pub mod types;
pub mod ui;

#[cfg(target_os = "android")]
pub use android_logger;

pub mod overture {
    use crate::renderer::Camera;
    use crate::renderer::Model;
    use crate::renderer::Renderer;
    use crate::ui;
    use glutin::config::{Config, ConfigSurfaceTypes, ConfigTemplate, ConfigTemplateBuilder};
    use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentContext};
    use glutin::display::{Display, DisplayApiPreference};
    use glutin::prelude::*;
    use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
    use raw_window_handle::{
        HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
    };
    use std::num::NonZeroU32;
    use winit::event::{Event, WindowEvent};
    use winit::event_loop::{ControlFlow, EventLoop, EventLoopWindowTarget};

    #[cfg(not(target_os = "android"))]
    use winit::event::{ElementState, MouseButton, VirtualKeyCode};
    
    struct SurfaceState {
        window: winit::window::Window,
        surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
    }

    pub struct App {
        winsys_display: RawDisplayHandle,
        glutin_display: Option<Display>,
        surface_state: Option<SurfaceState>,
        context: Option<glutin::context::PossiblyCurrentContext>,
        render_state: Option<Renderer>,
    }

    impl App {
        fn new(winsys_display: RawDisplayHandle) -> Self {
            Self {
                winsys_display,
                glutin_display: None,
                surface_state: None,
                context: None,
                render_state: None,
            }
        }

        #[allow(unused_variables)]
        fn create_display(
            raw_display: RawDisplayHandle,
            raw_window_handle: RawWindowHandle,
        ) -> Display {
            #[cfg(target_os = "linux")]
            let preference = DisplayApiPreference::Egl;

            #[cfg(target_os = "android")]
            let preference = DisplayApiPreference::Egl;

            #[cfg(target_os = "macos")]
            let preference = DisplayApiPreference::Cgl;

            #[cfg(target_os = "windows")]
            let preference = DisplayApiPreference::Wgl(Some(raw_window_handle));

            unsafe { Display::new(raw_display, preference).unwrap() }
        }

        fn ensure_glutin_display(&mut self, window: &winit::window::Window) {
            if self.glutin_display.is_none() {
                let raw_window_handle = window.raw_window_handle();
                self.glutin_display =
                    Some(Self::create_display(self.winsys_display, raw_window_handle));
            }
        }

        fn create_compatible_gl_context(
            glutin_display: &Display,
            raw_window_handle: RawWindowHandle,
            config: &Config,
        ) -> NotCurrentContext {
            let context_attributes = ContextAttributesBuilder::new()
                .with_context_api(ContextApi::Gles(Some(glutin::context::Version::new(2, 0))))
                .with_debug(true)
                .build(Some(raw_window_handle));

            let fallback_context_attributes = ContextAttributesBuilder::new()
                .with_context_api(ContextApi::Gles(None))
                .build(Some(raw_window_handle));
            unsafe {
                glutin_display
                    .create_context(&config, &context_attributes)
                    .unwrap_or_else(|_| {
                        glutin_display
                            .create_context(config, &fallback_context_attributes)
                            .expect("failed to create context")
                    })
            }
        }

        pub fn config_template(raw_window_handle: RawWindowHandle) -> ConfigTemplate {
            let builder = ConfigTemplateBuilder::new()
                .with_alpha_size(8)
                .compatible_with_native_window(raw_window_handle)
                .with_surface_type(ConfigSurfaceTypes::WINDOW)
                .with_api(glutin::config::Api::GLES2);

            builder.build()
        }

        fn ensure_surface_and_context<T>(&mut self, event_loop: &EventLoopWindowTarget<T>) {
            let window = winit::window::WindowBuilder::new()
                .with_inner_size(winit::dpi::LogicalSize::new(960.0, 640.0))
                .with_min_inner_size(winit::dpi::LogicalSize::new(480.0, 320.0))
                .with_title("TRS_24 Window")
                .build(&event_loop)
                .unwrap();
            let raw_window_handle = window.raw_window_handle();

            self.ensure_glutin_display(&window);
            let glutin_display = self
                .glutin_display
                .as_ref()
                .expect("Can't ensure surface + context without a Glutin Display connection");

            let template = Self::config_template(raw_window_handle);
            let config = unsafe {
                glutin_display
                    .find_configs(template)
                    .unwrap()
                    .reduce(|accum, config| {
                        if config.num_samples() > accum.num_samples() {
                            config
                        } else {
                            accum
                        }
                    })
                    .unwrap()
            };
            println!("Picked a config with {} samples", config.num_samples());

            let (width, height): (u32, u32) = window.inner_size().into();
            let raw_window_handle = window.raw_window_handle();
            let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
                raw_window_handle,
                NonZeroU32::new(width).unwrap(),
                NonZeroU32::new(height).unwrap(),
            );
            let surface = unsafe {
                glutin_display
                    .create_window_surface(&config, &attrs)
                    .unwrap()
            };
            let surface_state = SurfaceState { window, surface };

            let prev_ctx = self.context.take();
            match prev_ctx {
                Some(ctx) => {
                    let not_current_context = ctx
                        .make_not_current()
                        .expect("Failed to make GL context not current");
                    self.context = Some(
                        not_current_context
                            .make_current(&surface_state.surface)
                            .expect("Failed to make GL context current"),
                    );
                }
                None => {
                    let not_current_context = Self::create_compatible_gl_context(
                        glutin_display,
                        raw_window_handle,
                        &config,
                    );
                    self.context = Some(
                        not_current_context
                            .make_current(&surface_state.surface)
                            .expect("Failed to make GL context current"),
                    );
                }
            }

            self.surface_state = Some(surface_state);
        }

        fn ensure_renderer(&mut self, models: &Vec<Model>, ui: &Vec<ui::Element>) {
            let glutin_display = self
                .glutin_display
                .as_ref()
                .expect("Can't ensure renderer without a Glutin Display connection");
            self.render_state
                .get_or_insert_with(|| Renderer::new(glutin_display, models, ui));
        }

        fn queue_redraw(&self) {
            if let Some(surface_state) = &self.surface_state {
                log::trace!("Making Redraw Request");
                surface_state.window.request_redraw();
            }
        }

        fn resume<T>(
            &mut self,
            event_loop: &EventLoopWindowTarget<T>,
            models: &Vec<Model>,
            ui: &Vec<ui::Element>,
        ) {
            log::trace!("Resumed, creating render state...");
            self.ensure_surface_and_context(event_loop);
            self.ensure_renderer(models, ui);
            self.queue_redraw();
        }

        pub fn run(
            event_loop: EventLoop<()>,
            world_color: RGBA,
            models: Vec<Model>,
            ui: Vec<ui::Element>,
        ) {
            log::trace!("Running mainloop...");

            #[cfg(not(target_os = "android"))]
            if std::env::var("MESA_GLES_VERSION_OVERRIDE").is_err() {
                // Fallback to GLES version 2.0 for test runs (mesa drivers particularly)
                std::env::set_var("MESA_GLES_VERSION_OVERRIDE", "2.0");
            }

            let raw_display = event_loop.raw_display_handle();
            let mut app = App::new(raw_display);

            let mut active_touch_events: Vec<winit::event::Touch> = Vec::new();

            let mut camera: Camera = Camera::new(0.0, 0.0);
            let mut allowed_to_set_camera: bool = true;

            #[cfg(not(target_os = "android"))]
            let mut left_mouse_button_pressed = false;

            event_loop.run(move |event, event_loop, control_flow| {
                log::trace!("Received Winit event: {event:?}");

                if let Some(ref surface_state) = app.surface_state {
                    let (width, height): (u32, u32) = surface_state.window.inner_size().into();
                    if allowed_to_set_camera {
                        camera = Camera::new(width as f32, height as f32);
                        allowed_to_set_camera = false;
                    }
                }

                *control_flow = ControlFlow::Poll;
                match event {
                    Event::Resumed => {
                        app.resume(event_loop, &models, &ui);
                    }
                    Event::Suspended => {
                        log::trace!("Suspended, dropping surface state...");
                        app.surface_state = None;
                    }
                    Event::RedrawRequested(_) => {
                        log::trace!("Handling Redraw Request");

                        if let Some(ref surface_state) = app.surface_state {
                            if let Some(ctx) = &app.context {
                                if let Some(ref mut renderer) = app.render_state {
                                    renderer.draw(&world_color, &camera);

                                    if let Err(err) = surface_state.surface.swap_buffers(ctx) {
                                        log::error!("Failed to swap buffers after render: {}", err);
                                    }
                                }
                                app.queue_redraw();
                            }
                        }
                    }
                    Event::WindowEvent {
                        event: WindowEvent::Touch(location),
                        ..
                    } => {
                        match location.phase {
                            winit::event::TouchPhase::Started => {
                                active_touch_events.push(location);
                            }
                            winit::event::TouchPhase::Ended => {
                                active_touch_events.retain(|&t| t.id != location.id);
                            }
                            _ => {}
                        }

                        if active_touch_events.len() == 2 {
                            println!("Two fingers used: {:?}", active_touch_events);
                        }
                    }

                    #[cfg(not(target_os = "android"))]
                    Event::WindowEvent { event, .. } => match event {
                        WindowEvent::KeyboardInput { input, .. } => {
                            if let Some(key) = input.virtual_keycode {
                                match key {
                                    VirtualKeyCode::W => {
                                        camera.position += camera.speed * camera.orientation;
                                    }
                                    VirtualKeyCode::A => {
                                        camera.position += camera.speed
                                            * -nalgebra_glm::normalize(&nalgebra_glm::cross(
                                                &camera.orientation,
                                                &camera.up,
                                            ))
                                    }
                                    VirtualKeyCode::S => {
                                        camera.position -= camera.speed * camera.orientation
                                    }
                                    VirtualKeyCode::D => {
                                        camera.position += camera.speed
                                            * nalgebra_glm::normalize(&nalgebra_glm::cross(
                                                &camera.orientation,
                                                &camera.up,
                                            ))
                                    }
                                    VirtualKeyCode::Space => {
                                        camera.position += camera.speed * camera.up
                                    }
                                    VirtualKeyCode::LControl => {
                                        camera.position -= camera.speed * camera.up
                                    }
                                    VirtualKeyCode::LShift => {
                                        camera.speed = if input.state == ElementState::Pressed {
                                            0.4
                                        } else {
                                            0.1
                                        };
                                    }
                                    _ => {}
                                }
                            }
                        }
                        WindowEvent::MouseInput { state, button, .. } => match (state, button) {
                            (ElementState::Pressed, MouseButton::Left) => {
                                if let Some(ref surface_state) = app.surface_state {
                                    surface_state.window.set_cursor_visible(false);
                                    if camera.first_click {
                                        surface_state
                                            .window
                                            .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                                            .unwrap();

                                        surface_state
                                            .window
                                            .set_cursor_position(
                                                winit::dpi::LogicalPosition::<f64>::from((
                                                    camera.width as f64 / 2.0,
                                                    camera.height as f64 / 2.0,
                                                )),
                                            )
                                            .unwrap();

                                        surface_state
                                            .window
                                            .set_cursor_grab(
                                                winit::window::CursorGrabMode::Confined,
                                            )
                                            .unwrap();
                                        camera.first_click = false;

                                        left_mouse_button_pressed = true;
                                    }
                                }
                            }
                            (ElementState::Released, MouseButton::Left) => {
                                if let Some(ref surface_state) = app.surface_state {
                                    surface_state
                                        .window
                                        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                                        .unwrap();
                                    surface_state.window.set_cursor_visible(true);
                                    camera.first_click = true;
                                    left_mouse_button_pressed = false;
                                }
                            }
                            _ => {}
                        },
                        WindowEvent::CursorMoved { position, .. } => {
                            if left_mouse_button_pressed {
                                if let Some(ref surface_state) = app.surface_state {
                                    let mouse_x: f32 = position.x as f32;
                                    let mouse_y: f32 = position.y as f32;

                                    let rot_x = camera.sensitivity
                                        * (mouse_y - (camera.height / 2.0))
                                        / camera.height;
                                    let rot_y = camera.sensitivity
                                        * (mouse_x - (camera.width / 2.0))
                                        / camera.width;

                                    let new_orientation = nalgebra_glm::rotate_vec3(
                                        &camera.orientation,
                                        -rot_x as f32,
                                        &nalgebra_glm::normalize(&nalgebra_glm::cross(
                                            &camera.orientation,
                                            &camera.up,
                                        )),
                                    );

                                    if (nalgebra_glm::angle(&new_orientation, &camera.up) - 90.0)
                                        .abs()
                                        <= 85.0
                                    {
                                        camera.orientation = new_orientation;
                                    }

                                    camera.orientation = nalgebra_glm::rotate_vec3(
                                        &camera.orientation,
                                        -rot_y as f32,
                                        &camera.up,
                                    );

                                    surface_state
                                        .window
                                        .set_cursor_grab(winit::window::CursorGrabMode::Locked)
                                        .unwrap();

                                    surface_state
                                        .window
                                        .set_cursor_position(
                                            winit::dpi::LogicalPosition::<f64>::from((
                                                camera.width as f64 / 2.0,
                                                camera.height as f64 / 2.0,
                                            )),
                                        )
                                        .unwrap();

                                    surface_state
                                        .window
                                        .set_cursor_grab(winit::window::CursorGrabMode::Confined)
                                        .unwrap();
                                }
                            }
                        }
                        WindowEvent::CloseRequested => {
                            *control_flow = ControlFlow::Exit;
                        }
                        _ => {}
                    },
                    _ => {}
                }
            });
        }
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

    #[derive(Clone)]
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

    pub use winit::event_loop::EventLoopBuilder;
    #[cfg(target_os = "android")]
    pub use winit::platform::android::activity::AndroidApp;
    #[cfg(target_os = "android")]
    pub use winit::platform::android::EventLoopBuilderExtAndroid;
}
