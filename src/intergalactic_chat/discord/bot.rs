use std::ops::Deref;
use std::str::from_utf8;

use crate::intergalactic_chat::discord::util::{build_reply_for_webhook, get_link_webhook};
use crate::Config;
use rumqttc::{AsyncClient, Event, Incoming, QoS};
use serenity::async_trait;
use serenity::builder::ParseValue;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::model::prelude::AttachmentType;
use serenity::model::webhook::Webhook;
use serenity::prelude::*;
use tokio::sync::broadcast;
use tokio::task;

pub struct DiscordHandler {
	pub mq_client: AsyncClient,
	pub mq_event_receiver: broadcast::Receiver<Event>,
	pub config: Config,
}

#[async_trait]
impl EventHandler for DiscordHandler {
	async fn ready(&self, context: Context, ready: Ready) {
		println!(
			r##"
{} connected to Discord!
Invite with: https://discord.com/api/oauth2/authorize?client_id={}&permissions=1789592463424&scope=bot
"##,
			ready.user.name, ready.application.id
		);

		let mut event_receiver = self.mq_event_receiver.resubscribe();
		let mut webhooks: Vec<Webhook> = Vec::new();

		for channel in &self.config.discord.channels {
			webhooks.push(
				get_link_webhook(
					ChannelId::from(channel.to_owned()),
					format!("Webhook for {}", ready.user.name).deref(),
					&context,
				)
				.await,
			);
		}

		// Process the event and ensure it's a valid Discord message. The loop
		// will simply return if a problem is found, as none of the issues are
		// unrecoverable.
		// TODO: This can definitely be done more efficiently!
		loop {
			let message = match event_receiver.recv().await {
				Ok(event) => match event {
					Event::Incoming(incoming) => match incoming {
						Incoming::Publish(publish) => match from_utf8(&publish.payload) {
							Ok(payload_str) => match serde_json::from_str::<Message>(payload_str) {
								Ok(message) => message,
								_ => continue,
							},
							_ => continue,
						},
						_ => continue,
					},
					_ => continue,
				},
				_ => continue,
			};

			// TODO: Maybe make this multi-threaded.
			for webhook in &webhooks {
				let message = message.to_owned();
				let context = context.to_owned();
				let webhook = webhook.to_owned();

				task::spawn(async move {
					if message.channel_id.as_u64() == webhook.channel_id.unwrap().as_u64() {
						return;
					}

					match {
						webhook
							.execute(&context, false, |m| {
								m.content(message.content);
								m.avatar_url(message.author.face());
								m.username(message.author.name);
								m.allowed_mentions(|am| am.parse(ParseValue::Users));
								m.add_files(message.attachments.iter().fold(
									Vec::<AttachmentType>::new(),
									|mut files, f| {
										files.push(AttachmentType::from(f.url.deref()));
										files
									},
								));

								// Add an embed for replies
								message.referenced_message.and_then(|rm| {
									Some(m.embeds(vec![build_reply_for_webhook(rm)]))
								});

								m
							})
							.await
					} {
						Ok(_) => (),
						Err(e) => println!("Error sending Discord message {e}"),
					};
				});
			}
		}
	}

	async fn message(&self, _: Context, message: Message) {
		// TODO: Look into a solution that doesn't ignore bots.
		// TODO: Make this more efficient maybe?
		if !self
			.config
			.discord
			.channels
			.contains(message.channel_id.as_u64())
		{
			return;
		} else if message.author.bot {
			return;
		}

		let mq_client = &self.mq_client;
		let message_json = match serde_json::to_string(&message) {
			Ok(json) => json,
			Err(_) => return,
		};

		// TODO: This function currently just serializes and sends the entire Message
		// as JSON, which can be optimized by removing unused fields.
		mq_client
			.publish(
				&self.config.mqtt.topic,
				QoS::ExactlyOnce,
				false,
				message_json,
			)
			.await
			.ok();
	}
}
