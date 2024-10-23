use std::{env, error, fs};

fn link_icon() -> Result<(), Box<dyn error::Error>> {
	if env::var("TARGET")? == "x86_64-pc-windows-msvc" {
		println!("cargo::rerun-if-changed=../icon/resources.res");
		if fs::exists("../icon/resources.res")? {
			println!("cargo::rustc-link-arg-bins=icon/resources.res");
		} else {
			return Err(
				"could not find `resources.res` file; fend will not have an app icon".into(),
			);
		}
	}
	Ok(())
}

fn main() {
	if let Err(e) = link_icon() {
		println!("cargo::warning={e}");
	}
}
