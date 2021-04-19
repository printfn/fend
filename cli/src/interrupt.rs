use std::{process, sync};

pub struct CtrlC {
    running: sync::Arc<sync::atomic::AtomicBool>,
}

impl fend_core::Interrupt for CtrlC {
    fn should_interrupt(&self) -> bool {
        let running = self.running.load(sync::atomic::Ordering::Relaxed);
        !running
    }
}

impl CtrlC {
    pub fn reset(&self) {
        self.running.store(true, sync::atomic::Ordering::SeqCst);
    }
}

pub fn register_handler() -> CtrlC {
    let interrupt = CtrlC {
        running: sync::Arc::new(sync::atomic::AtomicBool::new(true)),
    };

    let r = interrupt.running.clone();
    let handler = move || {
        if !r.load(sync::atomic::Ordering::SeqCst) {
            // we already pressed Ctrl+C, so now kill the program
            process::exit(1);
        }
        r.store(false, sync::atomic::Ordering::SeqCst);
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
