use std::ops::Deref;
use std::str::from_utf8;
use std::sync::Arc;

use crate::intergalactic_chat::discord::util::{
	execute_message_for_webhook, get_link_webhook, CachedRelated,
};
use crate::Config;
use rumqttc::{AsyncClient, Event, Incoming, QoS};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::model::prelude::{GuildId, MessageId, MessageUpdateEvent};
use serenity::model::webhook::Webhook;
use serenity::prelude::*;
use tokio::sync::broadcast;
use tokio::task;

use super::util::MessageCache;

pub struct DiscordHandler {
	pub mq_client: AsyncClient,
	pub mq_event_receiver: broadcast::Receiver<Event>,
	pub config: Config,
	pub message_cache: Arc<Mutex<MessageCache>>,
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

			self.message_cache.lock().await.push(message.id, Vec::new());

			for webhook in &webhooks {
				let message = message.to_owned();
				let context = context.to_owned();
				let webhook = webhook.to_owned();
				let message_cache = Arc::clone(&self.message_cache);
				let message_id = message.id;

				task::spawn(async move {
					let m = execute_message_for_webhook(message, &context, &webhook).await;

					match m {
						Ok(m) => match m {
							Some(m) => {
								message_cache.lock().await.push_into_v(
									message_id,
									CachedRelated {
										related_channel_id: m.channel_id,
										related_message_id: m.id,
										related_webhook_id: webhook.id,
									},
								);
								()
							}
							None => return,
						},
						Err(e) => println!("Error sending message {e}"),
					}
				});
			}
		}
	}

	async fn message_delete(
		&self, context: Context, _: ChannelId, deleted_message_id: MessageId, _: Option<GuildId>,
	) {
		let c = &self.message_cache.lock().await;
		let cache_value = c.cache.get_key_value(&deleted_message_id);

		let messages = match cache_value {
			Some(v) => v,
			None => return,
		};

		for i in messages.1 {
			i.related_channel_id
				.delete_message(&context, i.related_message_id)
				.await
				.expect("Error deleting discord message");
		}
	}

	async fn message_update(&self, context: Context, new_data: MessageUpdateEvent) {
		let new_content = match new_data.content {
			Some(v) => v,
			None => return,
		};

		let c = &self.message_cache.lock().await;
		let cache_value = c.cache.get_key_value(&new_data.id);

		let messages = match cache_value {
			Some(v) => v,
			None => return,
		};

		for i in messages.1 {
			Webhook::from_id(&context, i.related_webhook_id)
				.await
				.expect("Error getting webhook from id")
				.edit_message(&context, i.related_message_id, |m| {
					m.content(new_content.clone())
				})
				.await
				.expect("Error editing webhook message");
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
