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

macro_rules! impl_serde {
	($($typ: ty)+) => {
		$(
			impl Serialize for $typ {
				fn serialize(&self, write: &mut impl io::Write) -> FResult<()> {
					Ok(write.write_all(&self.to_be_bytes())?)
				}
			}
			impl Deserialize for $typ {
				fn deserialize(read: &mut impl io::Read) -> FResult<Self> {
					let mut buf = [0; std::mem::size_of::<$typ>()];
					read.read_exact(&mut buf[..])?;
					Ok(<$typ>::from_be_bytes(buf))
				}
			}
		) +
	};
}

impl_serde!(u8 i32 u64 usize);

impl Serialize for &str {
	fn serialize(&self, write: &mut impl io::Write) -> FResult<()> {
		self.len().serialize(write)?;
		self.as_bytes()
			.iter()
			.try_for_each(|&bit| bit.serialize(write))?;
		Ok(())
	}
}

impl Deserialize for String {
	fn deserialize(read: &mut impl io::Read) -> FResult<Self> {
		let len = usize::deserialize(read)?;
		let mut buf = Vec::with_capacity(len);
		for _ in 0..len {
			buf.push(u8::deserialize(read)?);
		}
		match Self::from_utf8(buf) {
			Ok(string) => Ok(string),
			Err(_) => Err(FendError::DeserializationError),
		}
	}
}

impl Serialize for bool {
	fn serialize(&self, write: &mut impl io::Write) -> FResult<()> {
		Ok(write.write_all(&[u8::from(*self)])?)
	}
}

impl Deserialize for bool {
	fn deserialize(read: &mut impl io::Read) -> FResult<Self> {
		let mut buf = [0; 1];
		read.read_exact(&mut buf[..])?;
		match buf[0] {
			0 => Ok(false),
			1 => Ok(true),
			_ => Err(FendError::DeserializationError),
		}
	}
}

/*
	pub(crate) fn serialize(&self, write: &mut impl io::Write) -> Result<(), FendError> {
		Ok(())
	}

	pub(crate) fn deserialize(read: &mut impl io::Read) -> Result<Self, FendError> {
	}
*/

// pub(crate) fn serialize_u8(value: u8, write: &mut impl io::Write) -> io::Result<()> {
// 	write.write_all(&[value])
// }

// pub(crate) fn deserialize_u8(read: &mut impl io::Read) -> io::Result<u8> {
// 	let mut buf = [0; 1];
// 	read.read_exact(&mut buf[..])?;
// 	Ok(u8::from_be_bytes(buf))
// }

// pub(crate) fn serialize_i32(value: i32, write: &mut impl io::Write) -> io::Result<()> {
// 	write.write_all(&value.to_be_bytes())
// }

// pub(crate) fn deserialize_i32(read: &mut impl io::Read) -> io::Result<i32> {
// 	let mut buf = [0; 4];
// 	read.read_exact(&mut buf[..])?;
// 	Ok(i32::from_be_bytes(buf))
// }

// pub(crate) fn serialize_u64(value: u64, write: &mut impl io::Write) -> io::Result<()> {
// 	write.write_all(&value.to_be_bytes())
// }

// pub(crate) fn deserialize_u64(read: &mut impl io::Read) -> io::Result<u64> {
// 	let mut buf = [0; 8];
// 	read.read_exact(&mut buf[..])?;
// 	Ok(u64::from_be_bytes(buf))
// }

// pub(crate) fn serialize_usize(value: usize, write: &mut impl io::Write) -> io::Result<()> {
// 	write.write_all(&value.to_be_bytes())
// }

// pub(crate) fn deserialize_usize(read: &mut impl io::Read) -> io::Result<usize> {
// 	let mut buf = [0; std::mem::size_of::<usize>()];
// 	read.read_exact(&mut buf[..])?;
// 	Ok(usize::from_be_bytes(buf))
// }

// pub(crate) fn serialize_string(value: &str, write: &mut impl io::Write) -> io::Result<()> {
// 	serialize_usize(value.len(), write)?;
// 	for &b in value.as_bytes() {
// 		serialize_u8(b, write)?;
// 	}
// 	Ok(())
// }

// pub(crate) fn deserialize_string(read: &mut impl io::Read) -> Result<String, FendError> {
// 	let len = deserialize_usize(read)?;
// 	let mut buf = Vec::with_capacity(len);
// 	for _ in 0..len {
// 		buf.push(deserialize_u8(read)?);
// 	}
// 	match String::from_utf8(buf) {
// 		Ok(string) => Ok(string),
// 		Err(_) => Err(FendError::DeserializationError),
// 	}
// }

// pub(crate) fn serialize_bool(value: bool, write: &mut impl io::Write) -> io::Result<()> {
// 	write.write_all(&[u8::from(value)])
// }

// pub(crate) fn deserialize_bool(read: &mut impl io::Read) -> Result<bool, FendError> {
// 	let mut buf = [0; 1];
// 	read.read_exact(&mut buf[..])?;
// 	match buf[0] {
// 		0 => Ok(false),
// 		1 => Ok(true),
// 		_ => Err(FendError::DeserializationError),
// 	}
// }
