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

// These two functions should be merged at some point, but that would be a breaking
// API change.

#[wasm_bindgen(js_name = evaluateFendWithTimeout)]
pub fn evaluate_fend_with_timeout_2(input: &str, timeout: u32) -> String {
    evaluate_fend_with_timeout(input, timeout)
}

#[wasm_bindgen]
pub fn evaluate_fend_with_timeout(input: &str, timeout: u32) -> String {
    let mut ctx = fend_core::Context::new();
    let date = js_sys::Date::new_0();
    ctx.override_current_unix_time_ms(date.get_time() as u64);
    ctx.override_timezone_offset(date.get_timezone_offset() as u64 * 60);
    let interrupt = TimeoutInterrupt::new_with_timeout(u128::from(timeout));
    match fend_core::evaluate_with_interrupt(input, &mut ctx, &interrupt) {
        Ok(res) => res.get_main_result().to_string(),
        Err(msg) => format!("Error: {}", msg),
    }
}
