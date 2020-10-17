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
    timeout: u128,
}

impl TimeoutInterrupt {
    fn new_with_timeout(timeout: u128) -> Self {
        Self {
            start: Instant::now(),
            timeout,
        }
    }
}

impl fend_core::Interrupt for TimeoutInterrupt {
    fn should_interrupt(&self) -> bool {
        Instant::now().duration_since(self.start).as_millis() > self.timeout
    }
}

#[wasm_bindgen]
pub fn initialise() {
    utils::set_panic_hook();
}

#[wasm_bindgen]
pub fn evaluate_fend_with_timeout(input: &str, timeout: u32) -> String {
    let mut ctx = fend_core::Context::new();
    let interrupt = TimeoutInterrupt::new_with_timeout(u128::from(timeout));
    match fend_core::evaluate_with_interrupt(input, &mut ctx, &interrupt) {
        Ok(res) => res.get_main_result().to_string(),
        Err(msg) => format!("Error: {}", msg),
    }
}
