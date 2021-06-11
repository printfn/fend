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
            let mut c = self.ctx.borrow_mut();
            fend_core::evaluate_with_interrupt(line, &mut c, int)
        } else {
            let mut ctx_clone = self.ctx.borrow().clone();
            fend_core::evaluate_with_interrupt(line, &mut ctx_clone, int)
        }
    }
}
