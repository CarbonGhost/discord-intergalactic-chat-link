use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;

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
				client_id: String::new(),
				broker_ip: String::new(),
				broker_port: 0,
				topic: String::new(),
			},
			discord: DiscordConfig {
				channel: 0000000000000000000,
				bot_id: 0000000000000000000,
				token: String::new(),
			},
		}
	}

	/// Attempts to read the config file (`path`). If the file does not exist
	/// it attempts to create a new one and write an empty config. Reads and
	/// then returns a [`Config`].
	///
	/// ## Panics
	///
	/// This function will panic if:
	///
	/// - There is an error opening the file.
	/// - There is an error creating the file.
	/// - There is an error serializing a new [`Config`] to TOML.
	/// - There is an error deserializing from the file.
	/// - There is an error writing the file.
	/// - There is an error reading the file.
	pub fn initialize(path: &str) -> Result<Config, ()> {
		let mut file = match File::open(path) {
			Ok(f) => f,
			Err(e) => match e.kind() {
				ErrorKind::NotFound => match File::create(path) {
					Ok(mut f) => match toml::to_string_pretty::<Config>(&Config::new()) {
						Ok(c) => {
							match f.write_all(c.as_bytes()) {
								Ok(_) => (),
								Err(e) => panic!("Error writing {path}: {e}"),
							}

							f
						}
						Err(e) => panic!("Error serializing: {e}"),
					},
					Err(e) => panic!("Error creating {path}: {e}"),
				},
				_ => panic!("Error opening {path}: {e}"),
			},
		};

		let mut buf = String::new();

		let config = match file.read_to_string(&mut buf) {
			Ok(_) => match toml::from_str::<Config>(&buf) {
				Ok(toml) => toml,
				Err(e) => panic!(
					"Error deserializing {path}: {e} - Make sure your configuration is valid"
				),
			},
			Err(e) => panic!("Error reading {path}: {e} - Make sure this file exists"),
		};

		Ok(config)
	}
}

/// Struct for configuring the MQTT client.
#[derive(Serialize, Deserialize, Clone)]
pub struct MqttConfig {
	pub client_id: String,
	pub broker_ip: String,
	pub broker_port: u16,
	pub topic: String,
}

/// Struct for configuring the Discord client.
#[derive(Serialize, Deserialize, Clone)]
pub struct DiscordConfig {
	pub channel: u64,
	pub bot_id: u64,
	pub token: String,
}
