use serde::{Deserialize, Serialize};
use std::{
	fs::File,
	io::{Read, Write},
};

/// Struct representing the bot's configuration.
#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
	pub mqtt: MqttConfig,
	pub discord: DiscordConfig,
}

impl Config {
	/// Creates a new empty [`Config`].
	pub fn new() -> Config {
		Config {
			mqtt: MqttConfig {
				client_id: "".to_owned(),
				broker_ip: "".to_owned(),
				broker_port: 0,
				topic: "".to_owned(),
			},
			discord: DiscordConfig {
				channel: 0000000000000000000,
				bot_id: 0000000000000000000,
				token: "".to_owned(),
			},
		}
	}

	/// Attempts to read the config file. If one does not exist it attempts to
	/// write a new one and return a [`Config`].
	///
	/// ## Panics
	///
	/// - Panics if [`Config`] cannot be serialized to TOML.
	/// - Panics if the config file cannot be created.
	/// - Panics if the config file cannot be written to.
	pub fn initialize(path: &str) -> Result<Config, ()> {
		let mut file = match File::open(path) {
			Ok(file) => file,
			Err(_) => match File::create(path) {
				Ok(mut new_file) => {
					let empty_config = toml::to_string(&Config::new()).unwrap();
					new_file.write_all(empty_config.as_bytes()).unwrap();
					new_file
				}
				Err(why) => panic!("{why}"),
			},
		};
		let mut file_content = String::new();

		match file.read_to_string(&mut file_content) {
			Ok(_) => match toml::from_str::<Config>(&file_content) {
				Ok(toml) => Ok(toml),
				Err(_) => Err(()),
			},
			Err(_) => Err(()),
		}
	}
}

#[derive(Serialize, Deserialize, Clone)]
pub struct MqttConfig {
	pub client_id: String,
	pub broker_ip: String,
	pub broker_port: u16,
	pub topic: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DiscordConfig {
	pub channel: u64,
	pub bot_id: u64,
	pub token: String,
}
