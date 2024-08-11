#![allow(dead_code, warnings)]

pub mod renderer;
pub mod types;
pub mod ui;

pub mod overture {
    use crate::renderer::Camera;
    use crate::renderer::Model;
    use crate::renderer::Renderer;
    use crate::types;
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
    use std::time::Instant;
    pub use winit::event::{ElementState, MouseButton, TouchPhase};
    use winit::event::{Event, WindowEvent};
    use winit::event_loop::ActiveEventLoop;
    pub use winit::event_loop::EventLoop;
    use winit::keyboard::PhysicalKey;
    pub use winit::keyboard::{Key, KeyCode};

    struct SurfaceState {
        window: winit::window::Window,
        surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
    }

    pub enum CustomEvent {
        Keyboard(Key, ElementState, Box<dyn Fn(&mut Scene)>),
        Mouse(MouseButton, ElementState, Box<dyn Fn(&mut Scene)>),
        Touch(TouchPhase, Box<dyn Fn(&mut Scene)>),
    }

    pub struct Scene {
        event_loop: Option<EventLoop<CustomEvent>>,
        winsys_display: Option<RawDisplayHandle>,
        glutin_display: Option<Display>,
        surface_state: Option<SurfaceState>,
        context: Option<glutin::context::PossiblyCurrentContext>,
        pub render_state: Option<Renderer>,
    }

    impl Scene {
        pub fn new(event_loop: EventLoop<CustomEvent>) -> Self {
            #[cfg(not(target_os = "android"))]
            if std::env::var("MESA_GLES_VERSION_OVERRIDE").is_err() {
                // Fallback to GLES version 2.0 for test runs (mesa drivers particularly)
                std::env::set_var("MESA_GLES_VERSION_OVERRIDE", "2.0");
            }

            let winsys_display = event_loop.raw_display_handle().unwrap();
            Self {
                event_loop: Some(event_loop),
                winsys_display: Some(winsys_display),
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
                let raw_window_handle = window.raw_window_handle().unwrap();
                self.glutin_display = Some(Self::create_display(
                    self.winsys_display.unwrap(),
                    raw_window_handle,
                ));
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

        fn config_template(raw_window_handle: RawWindowHandle) -> ConfigTemplate {
            let builder = ConfigTemplateBuilder::new()
                .with_alpha_size(8)
                .compatible_with_native_window(raw_window_handle)
                .with_surface_type(ConfigSurfaceTypes::WINDOW)
                .with_api(glutin::config::Api::GLES2);

            builder.build()
        }

        fn ensure_surface_and_context(&mut self, event_loop: &ActiveEventLoop) {
            let window_attributes = winit::window::WindowAttributes::default()
                .with_inner_size(winit::dpi::LogicalSize::new(960.0, 640.0))
                .with_min_inner_size(winit::dpi::LogicalSize::new(480.0, 320.0))
                .with_title("TRS_24 Window");
            let window = event_loop.create_window(window_attributes).unwrap();
            let window_handle = window.raw_window_handle().unwrap();

            self.ensure_glutin_display(&window);
            let glutin_display = self
                .glutin_display
                .as_ref()
                .expect("Can't ensure surface + context without a Glutin Display connection");

            let template = Self::config_template(window_handle);
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
            let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
                window_handle,
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
                    let not_current_context =
                        Self::create_compatible_gl_context(glutin_display, window_handle, &config);
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
                surface_state.window.request_redraw();
            }
        }

        fn resume(
            &mut self,
            event_loop: &ActiveEventLoop,
            models: &Vec<Model>,
            ui: &Vec<ui::Element>,
        ) {
            self.ensure_surface_and_context(event_loop);
            self.ensure_renderer(models, ui);
            self.queue_redraw();
        }

        pub fn run(
            mut self,
            world_color: types::RGBA,
            model_pipeline: Vec<Model>,
            ui_pipeline: Vec<ui::Element>,
        ) {
            let mut camera: Camera = Camera::new(0.0, 0.0);
            let mut allowed_to_set_camera: bool = true;

            #[cfg(debug_assertions)]
            let mut left_mouse_button_pressed = false;

            let mut key_input_vec: Vec<(Key, ElementState, Box<dyn Fn(&mut Scene)>)> = Vec::new();
            let mut mouse_input_vec: Vec<(MouseButton, ElementState, Box<dyn Fn(&mut Scene)>)> =
                Vec::new();
            let mut touch_input_vec: Vec<(TouchPhase, Box<dyn Fn(&mut Scene)>)> = Vec::new();

            // let mut previous_frame_time = Instant::now();

            let mut imgui = imgui::Context::create();
            let mut platform = imgui_winit_support::WinitPlatform::init(&mut imgui);

            imgui.fonts().add_font(&[imgui::FontSource::TtfData {
                data: include_bytes!("../assets/fonts/Antonio-Bold.ttf"),
                size_pixels: 13.0,
                config: Some(imgui::FontConfig {
                    rasterizer_multiply: 1.5,
                    oversample_h: 4,
                    oversample_v: 4,
                    ..imgui::FontConfig::default()
                }),
            }]);

            imgui.fonts().build_alpha8_texture();
            imgui.fonts().build_rgba32_texture();

            imgui.set_ini_filename(None);

            if let Some(event_loop) = self.event_loop.take() {
                let _ = event_loop.run(move |event, event_loop| {
                    if let Some(ref surface_state) = self.surface_state {
                        let (width, height): (u32, u32) = surface_state.window.inner_size().into();
                        if allowed_to_set_camera {
                            camera = Camera::new(width as f32, height as f32);
                            allowed_to_set_camera = false;
                        }

                        platform.attach_window(
                            imgui.io_mut(),
                            &surface_state.window,
                            imgui_winit_support::HiDpiMode::Rounded,
                        );
                    }

                    match event {
                        Event::Resumed => {
                            self.resume(&event_loop, &model_pipeline, &ui_pipeline);
                        }
                        Event::Suspended => {
                            self.surface_state = None;
                        }
                        Event::WindowEvent {
                            event: WindowEvent::Touch(location),
                            ..
                        } => {
                            for phase in &touch_input_vec {
                                if location.phase == phase.0 {
                                    (phase.1)(&mut self);
                                }
                            }
                        }
                        Event::WindowEvent { event, .. } => match event {
                            WindowEvent::RedrawRequested => {
                                // let current_frame_time = std::time::Instant::now();
                                // let timestep = current_frame_time
                                //     .duration_since(previous_frame_time)
                                //     .as_secs_f32();
                                // previous_frame_time = current_frame_time;

                                // println!("{:?}ms", timestep * 1000.0);

                                if let Some(ref surface_state) = self.surface_state {
                                    if imgui.fonts().is_built() {
                                        let ui = imgui.frame();
                                        ui.show_demo_window(&mut true);
                                        platform.prepare_render(ui, &surface_state.window);
                                        imgui.render();
                                    }

                                    if let Some(ctx) = &self.context {
                                        if let Some(ref mut renderer) = self.render_state {
                                            renderer.draw(&world_color, &camera);

                                            if let Err(err) =
                                                surface_state.surface.swap_buffers(ctx)
                                            {
                                                println!(
                                                    "Failed to swap buffers after render: {}",
                                                    err
                                                );
                                            }
                                        }
                                        self.queue_redraw();
                                    }
                                }
                            }
                            WindowEvent::KeyboardInput { event, .. } => {
                                #[cfg(debug_assertions)]
                                match event.physical_key {
                                    PhysicalKey::Code(key_code) => match key_code {
                                        KeyCode::KeyW => {
                                            camera.position += camera.speed * camera.orientation;
                                        }
                                        KeyCode::KeyA => {
                                            camera.position += camera.speed
                                                * -nalgebra_glm::normalize(&nalgebra_glm::cross(
                                                    &camera.orientation,
                                                    &camera.up,
                                                ))
                                        }
                                        KeyCode::KeyS => {
                                            camera.position -= camera.speed * camera.orientation
                                        }
                                        KeyCode::KeyD => {
                                            camera.position += camera.speed
                                                * nalgebra_glm::normalize(&nalgebra_glm::cross(
                                                    &camera.orientation,
                                                    &camera.up,
                                                ))
                                        }
                                        KeyCode::Space => {
                                            camera.position += camera.speed * camera.up
                                        }
                                        KeyCode::ControlLeft => {
                                            camera.position -= camera.speed * camera.up
                                        }
                                        KeyCode::ShiftLeft => {
                                            camera.speed = if event.state == ElementState::Pressed {
                                                0.4
                                            } else {
                                                0.1
                                            };
                                        }
                                        _ => (),
                                    },
                                    _ => {}
                                }

                                // for given_key in &key_input_vec {
                                //     match (event.state, event.logical_key) {
                                //         (s, b) if s == given_key.1 && b == given_key.0 => {
                                //             (given_key.2)(&mut self);
                                //         }
                                //         _ => {}
                                //     }
                                // }
                            }
                            WindowEvent::MouseInput { state, button, .. } => {
                                #[cfg(debug_assertions)]
                                match (state, button) {
                                    (ElementState::Pressed, MouseButton::Left) => {
                                        if let Some(ref surface_state) = &self.surface_state {
                                            surface_state.window.set_cursor_visible(false);
                                            if camera.first_click {
                                                surface_state
                                                    .window
                                                    .set_cursor_grab(
                                                        winit::window::CursorGrabMode::Locked,
                                                    )
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
                                        if let Some(ref surface_state) = &self.surface_state {
                                            surface_state
                                                .window
                                                .set_cursor_grab(
                                                    winit::window::CursorGrabMode::Confined,
                                                )
                                                .unwrap();
                                            surface_state.window.set_cursor_visible(true);
                                            camera.first_click = true;
                                            left_mouse_button_pressed = false;
                                        }
                                    }

                                    _ => {}
                                }

                                for given_mouse in &mouse_input_vec {
                                    match (state, button) {
                                        (s, b) if s == given_mouse.1 && b == given_mouse.0 => {
                                            (given_mouse.2)(&mut self);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            #[cfg(debug_assertions)]
                            WindowEvent::CursorMoved { position, .. } => {
                                if left_mouse_button_pressed {
                                    if let Some(ref surface_state) = &self.surface_state {
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

                                        if (nalgebra_glm::angle(&new_orientation, &camera.up)
                                            - 90.0)
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
                                            .set_cursor_grab(
                                                winit::window::CursorGrabMode::Confined,
                                            )
                                            .unwrap();
                                    }
                                }
                            }
                            WindowEvent::CloseRequested => {
                                event_loop.exit();
                            }
                            _ => {}
                        },
                        Event::UserEvent(custom_event) => match custom_event {
                            CustomEvent::Keyboard(key, state, result) => {
                                key_input_vec.push((key, state, result))
                            }
                            CustomEvent::Touch(touch, result) => {
                                touch_input_vec.push((touch, result))
                            }
                            CustomEvent::Mouse(mouse, state, result) => {
                                mouse_input_vec.push((mouse, state, result))
                            }
                        },
                        _ => (),
                    }
                });
            }
        }
    }

    #[cfg(target_os = "android")]
    pub use winit::platform::android::activity::AndroidApp;
    #[cfg(target_os = "android")]
    pub use winit::platform::android::EventLoopBuilderExtAndroid;
}
