use std::{fs::{self, File}, io::{BufReader, BufWriter}, path::PathBuf, sync::Mutex};

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

use crate::matrix::set_error_message;

lazy_static! {
    pub static ref SAVING: Mutex<Saving> = Mutex::new(Saving::new());
}

pub const BUNDLE_ID: &str = "com.iewnfod.matrix.tui";
const SAVE_FILE_NAME: &str = "saves.json";

#[cfg(target_os = "macos")]
fn get_save_path() -> PathBuf {
	let mut path = dirs::home_dir().unwrap();
	path.push("Library");
	path.push("Application Support");
	path.push(BUNDLE_ID);
	path
}

#[cfg(not(target_os = "macos"))]
fn get_save_path() -> PathBuf {
	let mut path = dirs::home_dir().unwrap();
	path.push(".config");
	path.push(BUNDLE_ID);
	path
}

fn get_save_file_path() -> PathBuf {
	let mut path = get_save_path();
	path.push(SAVE_FILE_NAME);
	path
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Saving {
	pub token: String,
	pub username: String,
	pub server: String,
}

impl Saving {
	pub fn new() -> Self {
		if let Some(s) = Self::from_saves() {
			s
		} else {
			Self::default()
		}
	}

	fn from_saves() -> Option<Self> {
		let save_path = get_save_file_path();
		if !save_path.exists() {
			return None;
		}

		let file = File::open(save_path).unwrap();
		let reader = BufReader::new(file);

		let deserialized: Self = match serde_json::from_reader(reader) {
			Ok(d) => d,
			Err(e) => {
				set_error_message(format!("Failed to load savings: {}", e.to_string()));
				return None;
			}
		};

		Some(deserialized)
	}

	pub fn save(&self) {
		let p = get_save_file_path();
		if !p.exists() {
			fs::create_dir_all(p.parent().unwrap()).unwrap();
		}

		let file = match File::create(p) {
			Ok(f) => f,
			Err(e) => {
				set_error_message(format!("Error creating save file: {}", e));
				return;
			}
		};
		let writer = BufWriter::new(file);

		serde_json::to_writer_pretty(writer, self).unwrap();
	}
}
