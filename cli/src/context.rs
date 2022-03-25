use std::cell::RefCell;
use std::time;

pub struct HintInterrupt {
    start: time::Instant,
    duration: time::Duration,
}

impl fend_core::Interrupt for HintInterrupt {
    fn should_interrupt(&self) -> bool {
        time::Instant::now().duration_since(self.start) >= self.duration
    }
}

impl Default for HintInterrupt {
    fn default() -> Self {
        Self {
            start: time::Instant::now(),
            duration: time::Duration::from_millis(20),
        }
    }
}

#[derive(Clone)]
pub struct Context<'a> {
    ctx: &'a RefCell<fend_core::Context>,
}

impl<'a> Context<'a> {
    pub fn new(ctx: &'a RefCell<fend_core::Context>) -> Self {
        Self { ctx }
    }

    pub fn eval(
        &self,
        line: &str,
        int: &impl fend_core::Interrupt,
    ) -> Result<fend_core::FendResult, String> {
        let mut ctx_borrow = self.ctx.borrow_mut();
        ctx_borrow.set_random_u32_fn(random_u32);
        ctx_borrow.set_output_mode_terminal();
        fend_core::evaluate_with_interrupt(line, &mut ctx_borrow, int)
    }

    pub fn eval_hint(&self, line: &str) -> fend_core::FendResult {
        let mut ctx_borrow = self.ctx.borrow_mut();
        ctx_borrow.set_output_mode_terminal();
        let int = HintInterrupt::default();
        fend_core::evaluate_hint_with_interrupt(line, &mut ctx_borrow, &int)
    }
}

fn random_u32() -> u32 {
    let mut rng = nanorand::WyRand::new();
    nanorand::Rng::generate(&mut rng)
}
