mod sdf;
pub use sdf::*;

#[cfg(feature = "wasm")]
mod wasm {
    use crate::{render_scene, scene};
    use wasm_bindgen::prelude::wasm_bindgen;
    use web_sys::HtmlPreElement;

    #[wasm_bindgen]
    pub fn wasm_render(
        time: f32,
        screen_width: usize,
        screen_height: usize,
        pre: Option<HtmlPreElement>,
    ) -> f32 {
        let Some(pre) = pre else { return -1.0 };
        let scene = scene(time);
        let buffer = render_scene(&scene, screen_width, screen_height, 0.5);
        pre.set_inner_html(&buffer);
        time
    }
}
