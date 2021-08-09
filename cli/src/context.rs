use std::cell::RefCell;

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
        keep_results: bool,
        int: &impl fend_core::Interrupt,
    ) -> Result<fend_core::FendResult, String> {
        if keep_results {
            let mut ctx_borrow = self.ctx.borrow_mut();
            ctx_borrow.set_random_u32_fn(random_u32);
            ctx_borrow.set_output_mode_terminal();
            fend_core::evaluate_with_interrupt(line, &mut ctx_borrow, int)
        } else {
            let mut ctx_clone = self.ctx.borrow().clone();
            ctx_clone.disable_rng();
            ctx_clone.set_output_mode_terminal();
            fend_core::evaluate_with_interrupt(line, &mut ctx_clone, int)
        }
    }
}

fn random_u32() -> u32 {
    rand::random()
}
