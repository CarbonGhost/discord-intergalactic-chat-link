use std::time::Duration;

use crate::intergalactic_chat::bot::DiscordHandler;
use intergalactic_chat::config::Config;
use rumqttc::{AsyncClient, Event, MqttOptions, QoS};
use serenity::prelude::*;
use tokio;
use tokio::sync::broadcast;
use tokio::task;

mod intergalactic_chat;

#[tokio::main]
async fn main() {
	let config = Config::initialize("config.toml").expect("Failed to initialize config");

	// Setup the MQTT client
	let mut mq_options = MqttOptions::new(
		&config.mqtt.client_id,
		&config.mqtt.broker_ip,
		config.mqtt.broker_port,
	);
	mq_options.set_keep_alive(Duration::from_secs(5));
	let (mq_client, mut mq_event_loop) = AsyncClient::new(mq_options, 10);

	// Create a Tokio channel for receiving messages across threads
	// Start the MQTT client on a separate thread and return it
	let (event_sender, event_receiver) = broadcast::channel::<Event>(10);
	let topic = config.mqtt.topic.clone();

	let mq_client = task::spawn(async move {
		mq_client
			.subscribe(topic, QoS::AtMostOnce)
			.await
			.expect("Error creating MQTT subscription");

		// Start a new thread to continually advance the event loop
		task::spawn(async move {
			loop {
				let event = mq_event_loop.poll().await;

				match &event {
					Ok(v) => {
						event_sender.send(v.clone()).unwrap();
					}
					Err(e) => {
						println!("Error = {e:?}");
					}
				};
			}
		});

		mq_client
	})
	.await;

	// Setup the Discord client
	let intents = GatewayIntents::GUILD_MESSAGES
		| GatewayIntents::DIRECT_MESSAGES
		| GatewayIntents::MESSAGE_CONTENT;
	let mut discord_client = Client::builder(&config.discord.token, intents)
		.event_handler(DiscordHandler {
			mq_client: mq_client.expect("Error joining threads"),
			mq_event_receiver: event_receiver,
			config: config,
		})
		.await
		.expect("Error creating client");

	// Start the Discord client on the main thread
	match discord_client.start().await {
		Ok(_) => (),
		Err(why) => todo!("Can't start Discord client {:?}", why),
	}
}
