use std::time::Duration;

use env_file_reader::read_file;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tokio;
use tokio::task;

#[tokio::main]
async fn main() {
	// Read the bot token from the enviornment
	let env_vars = read_file(".env")
		.expect("Error reading enviornment variables. Do you have a \".env\" file?");
	let token = &env_vars["DISCORD_TOKEN"];

	// Setup the MQTT client
	let mut mq_options = MqttOptions::new("itergalactic_chat_client", "broker.hivemq.com", 1883);
	mq_options.set_keep_alive(Duration::from_secs(5));
	let (mq_client, mut mq_event_loop) = AsyncClient::new(mq_options, 10);

	// Start the MQTT client on a seperate thread and return it
	let mq_client = task::spawn(async move {
		mq_client
			.subscribe("intergalactic_chat/test", QoS::AtMostOnce)
			.await
			.expect("Error creaating MQTT subscription");

		// Start a new thread to continually advance the event loop
		task::spawn(async move {
			loop {
				let event = mq_event_loop.poll().await;

				match &event {
					Ok(v) => {
						println!("Event = {v:?}");
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
		.event_handler(Handler {
			mq_client: mq_client.expect("Error joining threads"),
		})
		.await
		.expect("Error creating client");

	// Start the Discord client on the main thread
	match discord_client.start().await {
		Ok(_) => (),
		Err(why) => todo!("Can't start Discord client {:?}", why),
	}
}

struct Handler {
	mq_client: AsyncClient,
}

#[async_trait]
impl EventHandler for Handler {
	async fn ready(&self, _: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);
	}

	async fn message(&self, ctx: Context, msg: Message) {
		let mq_client = &self.mq_client;

		println!("Discord message: {}", &msg.content);

		mq_client
			.publish(
				"intergalactic_chat/test",
				QoS::ExactlyOnce,
				false,
				msg.content,
			)
			.await
			.expect("Error publishing MQTT message");
	}
}
