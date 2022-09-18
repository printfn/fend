use std::{cell::RefCell, time};

use crate::config;

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

pub struct InnerCtx {
    core_ctx: fend_core::Context,

    // true if the user typed some partial input, false otherwise
    input_typed: bool,
}

impl InnerCtx {
    pub fn new(config: &config::Config) -> Self {
        let mut res = Self {
            core_ctx: fend_core::Context::new(),
            input_typed: false,
        };
        if config.coulomb_and_farad {
            res.core_ctx.use_coulomb_and_farad();
        }
        res
    }
}

#[derive(Clone)]
pub struct Context<'a> {
    ctx: &'a RefCell<InnerCtx>,
}

impl<'a> Context<'a> {
    pub fn new(ctx: &'a RefCell<InnerCtx>) -> Self {
        Self { ctx }
    }

    pub fn eval(
        &self,
        line: &str,
        int: &impl fend_core::Interrupt,
    ) -> Result<fend_core::FendResult, String> {
        let mut ctx_borrow = self.ctx.borrow_mut();
        ctx_borrow.core_ctx.set_random_u32_fn(random_u32);
        ctx_borrow.core_ctx.set_output_mode_terminal();
        ctx_borrow
            .core_ctx
            .set_exchange_rate_handler_v1(crate::exchange_rates::exchange_rate_handler);
        ctx_borrow.input_typed = false;
        fend_core::evaluate_with_interrupt(line, &mut ctx_borrow.core_ctx, int)
    }

    pub fn eval_hint(&self, line: &str) -> fend_core::FendResult {
        let mut ctx_borrow = self.ctx.borrow_mut();
        ctx_borrow.core_ctx.set_output_mode_terminal();
        ctx_borrow.input_typed = !line.is_empty();
        let int = HintInterrupt::default();
        fend_core::evaluate_preview_with_interrupt(line, &mut ctx_borrow.core_ctx, &int)
    }

    pub fn serialize(&self) -> Result<Vec<u8>, String> {
        let mut result = vec![];
        self.ctx
            .borrow()
            .core_ctx
            .serialize_variables(&mut result)?;
        Ok(result)
    }

    pub fn get_input_typed(&self) -> bool {
        self.ctx.borrow().input_typed
    }
}

fn random_u32() -> u32 {
    let mut rng = nanorand::WyRand::new();
    nanorand::Rng::generate(&mut rng)
}
