#![allow(clippy::module_name_repetitions)]

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct CtrlCInterrupt {
    running: Arc<AtomicBool>,
}

impl fend_core::Interrupt for CtrlCInterrupt {
    fn should_interrupt(&self) -> bool {
        let running = self.running.load(Ordering::Relaxed);
        !running
    }
}

impl CtrlCInterrupt {
    pub fn reset(&self) {
        self.running.store(true, Ordering::SeqCst);
    }
}

pub fn register_handler() -> CtrlCInterrupt {
    let interrupt = CtrlCInterrupt {
        running: Arc::new(AtomicBool::new(true)),
    };

    let r = interrupt.running.clone();
    let handler = move || {
        if !r.load(Ordering::SeqCst) {
            // we already pressed Ctrl+C, so now kill the program
            std::process::exit(1);
        }
        r.store(false, Ordering::SeqCst);
    };
    if ctrlc::set_handler(handler).is_err() {
        eprintln!("Unable to set Ctrl-C handler")
    }

    interrupt
}

pub struct NeverInterrupt {}
impl fend_core::Interrupt for NeverInterrupt {
    fn should_interrupt(&self) -> bool {
        false
    }
}
impl Default for NeverInterrupt {
    fn default() -> Self {
        Self {}
    }
}
