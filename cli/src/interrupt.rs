use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

pub struct CtrlC {
    running: Arc<AtomicBool>,
}

impl fend_core::Interrupt for CtrlC {
    fn should_interrupt(&self) -> bool {
        let running = self.running.load(Ordering::Relaxed);
        !running
    }
}

impl CtrlC {
    pub fn reset(&self) {
        self.running.store(true, Ordering::SeqCst);
    }
}

pub fn register_handler() -> CtrlC {
    let interrupt = CtrlC {
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

pub struct Never {}
impl fend_core::Interrupt for Never {
    fn should_interrupt(&self) -> bool {
        false
    }
}
impl Default for Never {
    fn default() -> Self {
        Self {}
    }
}
