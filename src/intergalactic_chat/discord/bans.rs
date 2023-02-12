use std::{
	collections::HashMap,
	fs::OpenOptions,
	io::{self, Read, Write},
	ops::Deref,
};

use serde::{Deserialize, Serialize};
use serenity::model::{
	prelude::{GuildId, UserId},
	Timestamp,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BanList {
	pub list: HashMap<UserId, BanEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BanEntry {
	pub reason: String,
	pub executor: UserId,
	pub ban_origin: GuildId,
	pub timestamp: Timestamp,
}

impl BanList {
	pub fn new() -> Self {
		BanList {
			list: HashMap::new(),
		}
	}

	pub fn initialize(path: &str) -> Self {
		let mut buf = String::new();

		OpenOptions::new()
			.read(true)
			.write(true)
			.create(true)
			.open(path)
			.unwrap()
			.read_to_string(&mut buf)
			.unwrap();

		if buf.len() == 0 {
			Self::new()
		} else {
			serde_json::from_str::<Self>(buf.deref()).unwrap()
		}
	}

	/// Writes [`BanList`] to the file provided by `path`. If that file does
	/// not exist, a new one will be created.
	pub fn write_to_file(self, path: &str) -> Result<Self, io::Error> {
		OpenOptions::new()
			.read(true)
			.write(true)
			.create(true)
			.truncate(true)
			.open(path)
			.unwrap()
			.write_all(serde_json::to_string(&self).unwrap().as_bytes())
			.unwrap();

		Ok(self)
	}
}
