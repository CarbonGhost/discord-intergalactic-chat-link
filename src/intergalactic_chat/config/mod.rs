use serde::{Deserialize, Serialize};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::ErrorKind;
use std::io::Read;
use std::io::Write;
use std::ops::Deref;
use toml::toml;

/// Struct representing the bot's configuration.
#[derive(Serialize, Deserialize, Clone)]
pub struct Config {
	pub mqtt: Mqtt,
	pub discord: Discord,
}

impl Config {
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
		let mut buf = String::new();
		let default_config = String::from(
			r#"
# This is the configuration file for your bot, make sure it is valid
# before starting the bot.

# For help and more information about the bot go 
# to: https://github.com/CarbonGhost/discord-intergalactic-chat-link

[mqtt]
broker_ip = "localhost"				# The IP address of your broker server.
broker_port = 1883						# The port the server is using, by default "1883".
client_id = "bot"							# The client ID used to connect to the MQTT server.
topic = "example/topic"				# The topic you wish to send / receive messages through.

[discord]
bot_id = 0000000000000000000	# The application ID of your bot, found via the Discord Developer Portal.
# A list of channels IDs for channels you wish for the bot to link, 
# separated by commas.
# You can have any number of channels on any number of servers, but the
# bot must have access to them and be able to create a webhook.
channels = [
	0000000000000000000,
	0000000000000000000,
	0000000000000000000,
]
# The bot's token, found via the Discord Developer portal.
# If you are reporting an issue make sure to omit this value!
token = "XXXXXXXXXXXXXXXXXXXXXXXXXX.XXXXXX.XXXXXXXX-XXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
		"#,
		);
		let mut file = OpenOptions::new()
			.create(true)
			.read(true)
			.write(true)
			.open(path)
			.expect(&format!("Error trying to open {path}"));
		file.read_to_string(&mut buf)
			.expect(&format!("Error trying to read {path}"));

		if buf.len() == 0 {
			file.write_all(default_config.as_bytes())
				.expect(&format!("Error trying to write to {path}"));

			Ok(toml::from_str::<Self>(&default_config).expect("Your configuration is invalid, double check you have entered the correct information"))
		} else {
			Ok(toml::from_str::<Self>(buf.deref()).expect("Your configuration is invalid, double check you have entered the correct information"))
		}
	}
}

/// Struct for configuring the MQTT client.
#[derive(Serialize, Deserialize, Clone)]
pub struct Mqtt {
	pub client_id: String,
	pub broker_ip: String,
	pub broker_port: u16,
	pub topic: String,
}

/// Struct for configuring the Discord client.
#[derive(Serialize, Deserialize, Clone)]
pub struct Discord {
	pub channels: Vec<u64>,
	pub bot_id: u64,
	pub token: String,
}
