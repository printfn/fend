use crate::{error::FendError, result::FResult};
use std::io;

pub(crate) trait Serialize
where
	Self: Sized,
{
	fn serialize(&self, write: &mut impl io::Write) -> FResult<()>;
}

pub(crate) trait Deserialize
where
	Self: Sized,
{
	fn deserialize(read: &mut impl io::Read) -> FResult<Self>;
}

#[allow(clippy::cast_possible_truncation)]
fn serialize_int(abs: u64, or: u8, write: &mut impl io::Write) -> FResult<()> {
	match abs {
		0..=0x17 => write.write_all(&[abs as u8 | or])?,
		0x18..=0xff => write.write_all(&[0x18 | or, abs as u8])?,
		0x100..=0xffff => write.write_all(&[0x19 | or, (abs >> 8) as u8, abs as u8])?,
		0x10000..=0xffff_ffff => write.write_all(&[
			0x1a | or,
			(abs >> 24) as u8,
			(abs >> 16) as u8,
			(abs >> 8) as u8,
			abs as u8,
		])?,
		0x1_0000_0000..=0xffff_ffff_ffff_ffff => write.write_all(&[
			0x1b | or,
			(abs >> 56) as u8,
			(abs >> 48) as u8,
			(abs >> 40) as u8,
			(abs >> 32) as u8,
			(abs >> 24) as u8,
			(abs >> 16) as u8,
			(abs >> 8) as u8,
			abs as u8,
		])?,
	}
	Ok(())
}

fn deserialize_int(reader: &mut impl io::Read) -> FResult<(u8, u64)> {
	let mut r = || -> FResult<u8> {
		let mut buf = [0];
		reader.read_exact(&mut buf)?;
		Ok(buf[0])
	};
	let n = r()?;
	Ok((
		n,
		match n & 0x1f {
			0..=0x17 => (n & 0x1f).into(),
			0x18 => r()?.into(),
			0x19 => u16::from_be_bytes([r()?, r()?]).into(),
			0x1a => u32::from_be_bytes([r()?, r()?, r()?, r()?]).into(),
			0x1b => u64::from_be_bytes([r()?, r()?, r()?, r()?, r()?, r()?, r()?, r()?]),
			_ => return Err(FendError::DeserializationError),
		},
	))
}

macro_rules! impl_serde {
	($($typ: ty)+) => {
		$(
			impl Serialize for $typ {
				#[allow(unused_comparisons, clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_lossless)]
				fn serialize(&self, write: &mut impl io::Write) -> FResult<()> {
					Ok(if *self < 0 {
						serialize_int((-(*self as i128) - 1) as u64, 0x20, write)?
					} else {
						serialize_int(*self as u64, 0, write)?
					})
				}
			}
			impl Deserialize for $typ {
				#[allow(clippy::cast_possible_truncation)]
				fn deserialize(read: &mut impl io::Read) -> FResult<Self> {
					let (flag, result) = deserialize_int(read)?;
					match flag {
						0..=0x1f => (),
						0x20..=0x37 => return (-i128::from(flag) + 0x1f).try_into().map_err(|_| FendError::DeserializationError),
						0x38..=0x3b => return (-i128::from(result) - 1).try_into().map_err(|_| FendError::DeserializationError),
						_ => return Err(FendError::DeserializationError),
					}
					let result = <$typ>::try_from(result).map_err(|_| FendError::DeserializationError)?;
					Ok(result)
				}
			}
		) +
	};
}

impl_serde!(u8 i32 u64 usize);

impl Serialize for &str {
	fn serialize(&self, write: &mut impl io::Write) -> FResult<()> {
		serialize_int(self.len() as u64, 0x60, write)?;
		write.write_all(self.as_bytes())?;
		Ok(())
	}
}

impl Deserialize for String {
	fn deserialize(read: &mut impl io::Read) -> FResult<Self> {
		let (flag, len) = deserialize_int(read)?;
		if !matches!(flag, 0x60..=0x7f) {
			return Err(FendError::DeserializationError);
		}
		let mut buf = vec![0; usize::try_from(len).map_err(|_| FendError::DeserializationError)?];
		read.read_exact(&mut buf)?;
		match Self::from_utf8(buf) {
			Ok(string) => Ok(string),
			Err(_) => Err(FendError::DeserializationError),
		}
	}
}

impl Serialize for bool {
	fn serialize(&self, write: &mut impl io::Write) -> FResult<()> {
		Ok(write.write_all(&[if *self { 0xf5 } else { 0xf4 }])?)
	}
}

impl Deserialize for bool {
	fn deserialize(read: &mut impl io::Read) -> FResult<Self> {
		let mut buf = [0; 1];
		read.read_exact(&mut buf[..])?;
		match buf[0] {
			0xf4 => Ok(false),
			0xf5 => Ok(true),
			_ => Err(FendError::DeserializationError),
		}
	}
}

#[cfg(test)]
mod tests {
	use std::io;

	use crate::serialize::{Deserialize, Serialize};

	#[track_caller]
	fn test(i: i128, bytes: &[u8]) {
		let mut buf = vec![];
		i32::try_from(i).unwrap().serialize(&mut buf).unwrap();
		assert_eq!(&buf, bytes);
		let mut cursor = io::Cursor::new(bytes);
		let res = i32::deserialize(&mut cursor).unwrap();
		assert_eq!(res, i32::try_from(i).unwrap());
	}

	#[test]
	fn cbor() {
		test(0, &[0x00]);
		test(16, &[0x10]);
		test(23, &[0x17]);
		test(24, &[0x18, 0x18]);
		test(426_937, &[0x1a, 0x00, 0x06, 0x83, 0xb9]);
		test(-1, &[0x20]);
		test(-24, &[0x37]);
		test(-426_937, &[0x3a, 0x00, 0x06, 0x83, 0xb8]);
	}

	#[test]
	fn cbor_string() {
		let mut buf = vec![];
		"hello world".serialize(&mut buf).unwrap();
		assert_eq!(
			&buf,
			&[
				0x6b, 0x68, 0x65, 0x6c, 0x6c, 0x6f, 0x20, 0x77, 0x6f, 0x72, 0x6c, 0x64
			]
		);
		let mut cursor = io::Cursor::new(buf);
		let s = String::deserialize(&mut cursor).unwrap();
		assert_eq!(s, "hello world");
	}
}
