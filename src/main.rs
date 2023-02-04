use std::ops::Deref;
use std::time::Duration;

use env_file_reader::read_file;
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

const MQ_CLIENT_ID: &str = "itergalactic_chat_client";
const MQ_BROKER_IP: &str = "broker.hivemq.com";
const MQ_BROKER_PORT: u16 = 1883;
const MQ_TOPIC: &str = "intergalactic_chat/test";
const DISCORD_CHANNEL: u64 = 1071016502746157146;
const DISCORD_BOT_ID: u64 = 1071017546934915114;

#[tokio::main]
async fn main() {
	// Read the bot token from the enviornment
	let env_vars = read_file(".env")
		.expect("Error reading enviornment variables. Do you have a \".env\" file?");
	let token = &env_vars["DISCORD_TOKEN"];

	// Setup the MQTT client
	let mut mq_options = MqttOptions::new(MQ_CLIENT_ID, MQ_BROKER_IP, MQ_BROKER_PORT);
	mq_options.set_keep_alive(Duration::from_secs(5));
	let (mq_client, mut mq_event_loop) = AsyncClient::new(mq_options, 10);

	// Create a Tokio channel for recieving messages accross threads
	let (event_sender, mut event_receiver) = broadcast::channel::<Event>(10);

	// Start the MQTT client on a seperate thread and return it
	let mq_client = task::spawn(async move {
		mq_client
			.subscribe(MQ_TOPIC, QoS::AtMostOnce)
			.await
			.expect("Error creaating MQTT subscription");

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
	let mut discord_client = Client::builder(&token, intents)
		.event_handler(DiscordHandler {
			mq_client: mq_client.expect("Error joining threads"),
			mq_event_reciever: event_receiver,
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
	mq_event_reciever: broadcast::Receiver<Event>,
}

#[async_trait]
#[allow(unused_variables)]
impl EventHandler for DiscordHandler {
	async fn ready(&self, context: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);

		let mut event_receiver = self.mq_event_reciever.resubscribe();

		loop {
			let event = event_receiver.recv().await;

			match event {
				Ok(event) => match event {
					Event::Incoming(incoming) => match incoming {
						Incoming::Publish(publish) => {
							let s = std::str::from_utf8(publish.payload.deref())
								.expect("Encoding error");
							println!("Received: {s}");

							ChannelId::from(DISCORD_CHANNEL)
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
		if message.channel_id != ChannelId::from(DISCORD_CHANNEL) {
			return;
		} else if message.author.id == UserId::from(DISCORD_BOT_ID) {
			return;
		}

		let mq_client = &self.mq_client;

		println!("Discord message: {}", &message.content);

		mq_client
			.publish(MQ_TOPIC, QoS::ExactlyOnce, false, message.content)
			.await
			.expect("Error publishing MQTT message");
	}
}
