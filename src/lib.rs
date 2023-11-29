use std::num::NonZeroU32;

use raw_window_handle::{
    HasRawDisplayHandle, HasRawWindowHandle, RawDisplayHandle, RawWindowHandle,
};

use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopWindowTarget};
#[cfg(target_os = "android")]
use winit::platform::android::activity::AndroidApp;

use glutin::config::{Config, ConfigSurfaceTypes, ConfigTemplate, ConfigTemplateBuilder};
use glutin::context::{ContextApi, ContextAttributesBuilder, NotCurrentContext};
use glutin::display::{Display, DisplayApiPreference};
use glutin::prelude::*;
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};

mod renderer;
use crate::renderer::renderer::Renderer;

struct SurfaceState {
    window: winit::window::Window,
    surface: glutin::surface::Surface<glutin::surface::WindowSurface>,
}

struct App {
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
}

impl App {
    #[allow(unused_variables)]
    fn create_display(
        raw_display: RawDisplayHandle,
        raw_window_handle: RawWindowHandle,
    ) -> Display {
        unsafe { Display::new(raw_display, DisplayApiPreference::Egl).unwrap() }
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
        let window = winit::window::Window::new(&event_loop).unwrap();
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
                    // Find the config with the maximum number of samples.
                    //
                    // In general if you're not sure what you want in template you can request or
                    // don't want to require multisampling for example, you can search for a
                    // specific option you want afterwards.
                    //
                    // XXX however on macOS you can request only one config, so you should do
                    // a search with the help of `find_configs` and adjusting your template.
                    if config.num_samples() > accum.num_samples() {
                        config
                    } else {
                        accum
                    }
                })
                .unwrap()
        };
        println!("Picked a config with {} samples", config.num_samples());
        // XXX: Winit is missing a window.surface_size() API and the inner_size may be the wrong
        // size to use on some platforms!
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
                let not_current_context =
                    Self::create_compatible_gl_context(glutin_display, raw_window_handle, &config);
                self.context = Some(
                    not_current_context
                        .make_current(&surface_state.surface)
                        .expect("Failed to make GL context current"),
                );
            }
        }

        self.surface_state = Some(surface_state);
    }

    fn ensure_renderer(&mut self) {
        let glutin_display = self
            .glutin_display
            .as_ref()
            .expect("Can't ensure renderer without a Glutin Display connection");
        self.render_state
            .get_or_insert_with(|| Renderer::new(glutin_display));
    }

    fn queue_redraw(&self) {
        if let Some(surface_state) = &self.surface_state {
            log::trace!("Making Redraw Request");
            surface_state.window.request_redraw();
        }
    }

    fn resume<T>(&mut self, event_loop: &EventLoopWindowTarget<T>) {
        log::trace!("Resumed, creating render state...");
        self.ensure_surface_and_context(event_loop);
        self.ensure_renderer();
        self.queue_redraw();
    }
}

fn run(event_loop: EventLoop<()>) {
    log::trace!("Running mainloop...");

    let raw_display = event_loop.raw_display_handle();
    let mut app = App::new(raw_display);
    let mut active_touch_events: Vec<winit::event::Touch> = Vec::new();

    event_loop.run(move |event, event_loop, control_flow| {
        log::trace!("Received Winit event: {event:?}");

        *control_flow = ControlFlow::Wait;
        match event {
            Event::Resumed => {
                app.resume(event_loop);
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
                            let (width, height): (u32, u32) =
                                surface_state.window.inner_size().into();
                            renderer.draw(width.try_into().unwrap(), height.try_into().unwrap());
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
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => *control_flow = ControlFlow::Exit,
            _ => {}
        }
    });
}

#[cfg(target_os = "android")]
#[no_mangle]
fn android_main(app: AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(android_logger::Config::default());

    let event_loop = EventLoopBuilder::new().with_android_app(app).build();
    run(event_loop);
}

// declared as pub to avoid dead_code warnings from cdylib target build
#[cfg(not(target_os = "android"))]
pub fn main() {
    let event_loop = EventLoopBuilder::new().build();
    run(event_loop);
}
