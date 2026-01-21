use crate::state::State;
use log::{debug, error, info, warn};
use std::num::NonZeroUsize;
use std::sync::Arc;
use winit::application::ApplicationHandler;
use winit::dpi::PhysicalSize;
use winit::event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent};
#[allow(unused_imports)]
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::keyboard::{KeyCode, PhysicalKey};
use winit::window::{Window, WindowId};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub struct App {
    #[cfg(target_arch = "wasm32")]
    proxy: Option<winit::event_loop::EventLoopProxy<State>>,
    state: InitializationState,
}

#[derive(Clone)]
pub struct Configuration {
    pub tiles: NonZeroUsize,
}

enum InitializationState {
    Before(Configuration),
    After(State),
}

impl App {
    pub fn new(
        configuration: Configuration,
        #[cfg(target_arch = "wasm32")] event_loop: &EventLoop<State>,
    ) -> Self {
        #[cfg(target_arch = "wasm32")]
        let proxy = Some(event_loop.create_proxy());
        Self {
            #[cfg(target_arch = "wasm32")]
            proxy,
            state: InitializationState::Before(configuration),
        }
    }
}

impl ApplicationHandler<State> for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let InitializationState::Before(configuration) = &self.state else {
            debug!("Preventing double initialization");
            return;
        };
        let configuration = configuration.clone();

        #[allow(unused_mut)]
        let mut window_attributes = Window::default_attributes()
            .with_inner_size(PhysicalSize::new(800, 300));

        #[cfg(target_arch = "wasm32")]
        {
            use wasm_bindgen::JsCast;
            use winit::platform::web::WindowAttributesExtWebSys;

            const CANVAS_ID: &str = "canvas";

            let window = wgpu::web_sys::window().unwrap_throw();
            let document = window.document().unwrap_throw();
            let canvas = document.get_element_by_id(CANVAS_ID).unwrap_throw();
            let html_canvas_element = canvas.unchecked_into();
            window_attributes = window_attributes.with_canvas(Some(html_canvas_element));
        }

        let window = Arc::new(event_loop.create_window(window_attributes).unwrap());

        #[cfg(not(target_arch = "wasm32"))]
        {
            self.state = InitializationState::After(
                pollster::block_on(State::new(window.clone(), configuration)).unwrap(),
            )
        }

        #[cfg(target_arch = "wasm32")]
        {
            // Run the future asynchronously and use the
            // proxy to send the results to the event loop
            if let Some(proxy) = self.proxy.take() {
                wasm_bindgen_futures::spawn_local(async move {
                    assert!(
                        proxy
                            .send_event(
                                State::new(window, configuration)
                                    .await
                                    .expect("Unable to create canvas!!!")
                            )
                            .is_ok()
                    )
                });
            }
        }
    }

    #[allow(unused_mut)]
    #[cfg(target_arch = "wasm32")]
    fn user_event(&mut self, _event_loop: &ActiveEventLoop, mut state: State) {
        {
            state.window.request_redraw();
            state.resize(
                state.window.inner_size().width,
                state.window.inner_size().height,
            );
        }

        self.state = InitializationState::After(state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let InitializationState::After(state) = &mut self.state else {
            return;
        };

        if state.on_window_event(&event) {
            return;
        }
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(PhysicalSize { width, height }) => {
                state.resize(width, height);
            }
            WindowEvent::RedrawRequested => {
                state.update();
                match state.render() {
                    Ok(_) => {}
                    Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                        state.resize(state.config.width, state.config.height)
                    }
                    Err(e) => {
                        error!("render: {:?}", e);
                    }
                }
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key_code),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match key_code {
                KeyCode::Escape => event_loop.exit(),
                _ => {}
            },
            _ => {}
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        let InitializationState::After(state) = &mut self.state else {
            return;
        };

        state.on_device_event(&event);
    }
}

impl Default for Configuration {
    fn default() -> Self {
        Configuration {
            tiles: NonZeroUsize::new(128).unwrap(),
        }
    }
}
