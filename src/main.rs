use std::sync::Arc;
use std::time::Duration;

use crate::intergalactic_chat::discord::bot::DiscordHandler;
use crate::intergalactic_chat::discord::cache::MessageCache;
use intergalactic_chat::config::Config;
use intergalactic_chat::discord::bans::BanList;
use intergalactic_chat::mqtt::poll_event_loop;
use rumqttc::{AsyncClient, Event, MqttOptions, QoS};
use serenity::prelude::*;
use tokio::sync::broadcast;
use tokio::task;

mod intergalactic_chat;

#[tokio::main]
async fn main() {
	let config = Config::initialize("config.toml").expect("Failed to initialize the config");
	let message_cache = Arc::new(Mutex::new(MessageCache::initialize(".cache", 100)));
	let ban_list = Arc::new(Mutex::new(BanList::initialize(".bans")));

	let mut mq_options = MqttOptions::new(
		&config.mqtt.client_id,
		&config.mqtt.broker_ip,
		config.mqtt.broker_port,
	);
	mq_options.set_keep_alive(Duration::from_secs(5));
	let (mq_client, mq_event_loop) = AsyncClient::new(mq_options, 10);
	let (event_sender, event_receiver) = broadcast::channel::<Event>(10);
	let topic = config.mqtt.topic.clone();
	let mq_client = task::spawn(async move {
		mq_client
			.subscribe(topic, QoS::AtMostOnce)
			.await
			.expect("Error creating MQTT subscription");

		task::spawn(async move {
			poll_event_loop(mq_event_loop, event_sender).await;
		});

		mq_client
	})
	.await;

	let intents = GatewayIntents::GUILD_MESSAGES
		| GatewayIntents::DIRECT_MESSAGES
		| GatewayIntents::MESSAGE_CONTENT;
	let mut discord_client = Client::builder(&config.discord.token, intents)
		.event_handler(DiscordHandler {
			mq_client: mq_client.expect("Threading error"),
			mq_event_receiver: event_receiver,
			config,
			message_cache,
			ban_list,
		})
		.await
		.expect("Error creating Discord client");
	discord_client
		.start()
		.await
		.expect("Failed to start Discord client");
}
