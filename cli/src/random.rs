use nanorand::{Rng, WyRand};

#[derive(Debug)]
pub struct Random {
	rng: WyRand,
}

impl Random {
	pub fn new() -> Self {
		Self { rng: WyRand::new() }
	}
}

impl fend_core::Random for Random {
	fn random_u32(&mut self) -> u32 {
		self.rng.generate::<u32>()
	}
}
