use crate::app::{App, Configuration};
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;
use winit::event_loop::EventLoop;

pub mod app;
mod map;
mod state;

pub fn run(configuration: Configuration) -> anyhow::Result<()> {
    #[cfg(not(target_arch = "wasm32"))]
    {
        env_logger::init();
    }
    #[cfg(target_arch = "wasm32")]
    {
        console_log::init_with_level(log::Level::Info).unwrap_throw();
        use log::info;
        info!("Starting...");
    }

    let event_loop = EventLoop::with_user_event().build()?;
    let mut app = App::new(
        configuration,
        #[cfg(target_arch = "wasm32")]
        &event_loop,
    );
    event_loop.run_app(&mut app)?;

    Ok(())
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn run_web() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();

    let configuration = Configuration::default();
    run(configuration).unwrap_throw();

    Ok(())
}
