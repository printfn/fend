mod utils;

use instant::Instant;
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

struct TimeoutInterrupt {
    start: Instant,
}

impl TimeoutInterrupt {
    fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl fend_core::Interrupt for TimeoutInterrupt {
    fn should_interrupt(&self) -> bool {
        Instant::now().duration_since(self.start).as_millis() > 500
    }
}

#[wasm_bindgen]
pub fn initialise() {
    utils::set_panic_hook();
}

#[wasm_bindgen]
pub fn evaluate_fend(input: &str) -> String {
    let mut ctx = fend_core::Context::new();
    let interrupt = TimeoutInterrupt::new();
    match fend_core::evaluate_with_interrupt(input, &mut ctx, &interrupt) {
        Ok(res) => res.get_main_result().to_string(),
        Err(msg) => msg,
    }
}
