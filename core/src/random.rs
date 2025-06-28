/// This trait is fend's RNG.
pub trait Random: std::fmt::Debug {
	/// Generate a random u32.
	fn random_u32(&mut self) -> u32;
}
