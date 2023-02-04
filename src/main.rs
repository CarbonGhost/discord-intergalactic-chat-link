use std::ops::Deref;
use std::time::Duration;

use intergalactic_chat::config::Config;
use rumqttc::{AsyncClient, Event, Incoming, MqttOptions, QoS};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::model::prelude::UserId;
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

struct DiscordHandler {
	mq_client: AsyncClient,
	mq_event_receiver: broadcast::Receiver<Event>,
	config: Config,
}

#[async_trait]
#[allow(unused_variables)]
impl EventHandler for DiscordHandler {
	async fn ready(&self, context: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);

		let mut event_receiver = self.mq_event_receiver.resubscribe();

		loop {
			let event = event_receiver.recv().await;

			match event {
				Ok(event) => match event {
					Event::Incoming(incoming) => match incoming {
						Incoming::Publish(publish) => {
							let s = std::str::from_utf8(publish.payload.deref())
								.expect("Encoding error");
							println!("Received: {s}");

							ChannelId::from(self.config.discord.channel)
								.send_message(&context, |m| m.content(s))
								.await
								.expect("Error sending message to Discord");
						}
						_ => (),
					},
					_ => (),
				},
				Err(why) => println!("ERR: {why:?}"),
			}
		}
	}

	async fn message(&self, context: Context, message: Message) {
		if message.channel_id != ChannelId::from(self.config.discord.channel) {
			return;
		} else if message.author.id == UserId::from(self.config.discord.bot_id) {
			return;
		}

		let mq_client = &self.mq_client;

		println!("Discord message: {}", &message.content);

		mq_client
			.publish(
				&self.config.mqtt.topic,
				QoS::ExactlyOnce,
				false,
				message.content,
			)
			.await
			.expect("Error publishing MQTT message");
	}
}
