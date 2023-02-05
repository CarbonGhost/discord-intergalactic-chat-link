use std::ops::Deref;

use crate::Config;
use rumqttc::{AsyncClient, Event, Incoming, QoS};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::model::prelude::{AttachmentType, UserId};
use serenity::prelude::*;
use tokio;
use tokio::sync::broadcast;

pub struct DiscordHandler {
	pub mq_client: AsyncClient,
	pub mq_event_receiver: broadcast::Receiver<Event>,
	pub config: Config,
}

#[async_trait]
impl EventHandler for DiscordHandler {
	async fn ready(&self, context: Context, ready: Ready) {
		println!("{} is connected!", ready.user.name);

		let mut event_receiver = self.mq_event_receiver.resubscribe();

		// Watch for incoming events and only send a Discord message if they are
		// valid content.
		loop {
			let event = event_receiver.recv().await;

			match event {
				Ok(event) => match event {
					Event::Incoming(incoming) => match incoming {
						Incoming::Publish(publish) => {
							let s = std::str::from_utf8(publish.payload.deref())
								.expect("Error parsing bytes");

							match serde_json::from_str::<Message>(s) {
								Ok(v) => {
									println!("Received: {v:#?}");
									ChannelId::from(self.config.discord.channel)
										.send_message(&context, |m| {
											m.content(v.content);
											m.add_files(v.attachments.iter().fold(
												Vec::<AttachmentType>::new(),
												|mut acc, a| {
													let _ = &acc.push(AttachmentType::from(
														a.proxy_url.deref(),
													));
													acc
												},
											));

											m
										})
										.await
										.expect("Error sending message to Discord");
								}
								Err(_) => {
									println!("Error deserializing JSON");
								}
							};
						}
						_ => (),
					},
					_ => (),
				},
				Err(why) => println!("ERR: {why:?}"),
			}
		}
	}

	async fn message(&self, _: Context, message: Message) {
		if message.channel_id != ChannelId::from(self.config.discord.channel) {
			return;
		} else if message.author.id == UserId::from(self.config.discord.bot_id) {
			return;
		}

		let mq_client = &self.mq_client;

		println!("Discord message: {}", &message.content);

		// TODO: This function currently just serializes and sends the entire Message
		// as JSON, which can be optimized by removing unused fields.
		mq_client
			.publish(
				&self.config.mqtt.topic,
				QoS::ExactlyOnce,
				false,
				serde_json::to_string(&message).expect("Error serializing Discord message"),
			)
			.await
			.expect("Error publishing MQTT message");
	}
}
