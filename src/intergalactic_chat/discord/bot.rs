use std::ops::Deref;
use std::str::from_utf8;

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
		println!(
			"{} has successfully connected to the Discord gateway.",
			ready.user.name
		);

		let mut event_receiver = self.mq_event_receiver.resubscribe();

		loop {
			// Process the event and ensure it's a valid Discord message. The loop
			// will simply return if a problem is found, as none of the issues are
			// unrecoverable.
			// TODO: This can definitely be done more efficiently!
			let event = match event_receiver.recv().await {
				Ok(event) => event,
				Err(_) => continue,
			};

			let payload = match event {
				Event::Incoming(incoming) => match incoming {
					Incoming::Publish(publish) => publish.payload,
					_ => continue,
				},
				_ => continue,
			};

			let s = match from_utf8(&payload) {
				Ok(s) => s,
				Err(_) => continue,
			};

			let message = match serde_json::from_str::<Message>(s) {
				Ok(json) => json,
				Err(_) => continue,
			};

			println!("Received: {message:#?}");

			match {
				ChannelId::from(self.config.discord.channel)
					.send_message(&context, |m| {
						m.content(message.content);
						m.add_files(message.attachments.iter().fold(
							Vec::<AttachmentType>::new(),
							|mut files, attachment| {
								let _ =
									&files.push(AttachmentType::from(attachment.proxy_url.deref()));
								files
							},
						));

						m
					})
					.await
			} {
				Ok(_) => (),
				Err(e) => println!("Error sending Discord message {e}"),
			};
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
