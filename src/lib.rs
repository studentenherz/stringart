mod stringart;
pub mod utils;

#[cfg(target_arch = "wasm32")]
mod wasm {
    #[global_allocator]
    static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

    extern crate console_error_panic_hook;
    use crate::stringart;
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    pub fn generate_stringart(
        image_data: &[u8],
        num_points: usize,
        num_lines: usize,
        weight: u8,
    ) -> Result<JsValue, serde_wasm_bindgen::Error> {
        console_error_panic_hook::set_once();

        serde_wasm_bindgen::to_value(&stringart::generate_stringart(
            image_data, num_points, num_lines, weight,
        ))
    }
}
