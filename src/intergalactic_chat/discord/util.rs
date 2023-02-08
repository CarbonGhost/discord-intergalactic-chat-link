use std::{collections::HashMap, ops::Deref};

use serde::{Deserialize, Serialize};
use serenity::{
	builder::ParseValue,
	model::{
		prelude::{AttachmentType, ChannelId, Embed, Message, MessageId, WebhookId},
		webhook::Webhook,
	},
	prelude::Context,
};
use tokio::task;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageCache {
	/// The maximum number of entires the cache can hold before removing old ones.
	size: usize,
	/// A hash map representing the cached values.
	///
	/// The key represents the original message sent by the user and the value
	/// contains data related to messages the bot has posted.
	pub cache: HashMap<MessageId, Vec<CachedRelated>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CachedRelated {
	pub related_channel_id: ChannelId,
	pub related_message_id: MessageId,
	pub related_webhook_id: WebhookId,
}

impl MessageCache {
	pub fn new(size: usize) -> Self {
		Self {
			size,
			cache: HashMap::new(),
		}
	}

	pub fn push(&mut self, k: MessageId, v: Vec<CachedRelated>) -> &mut Self {
		// Remove 1 item from the map if it is larger than `size`
		if &self.cache.len() == &self.size {
			let keys: Vec<_> = { self.cache.keys().take(1).copied().collect() };

			for key in keys {
				self.cache.remove(&key);
			}
		}

		self.cache.insert(k, v);

		self
	}
}

/// Get the webhook for the linked channel, if the channel doesn't already
/// have one create a new one.
///
/// ## Panics
///
/// - Panics if the webhook cannot be created, because the bot won't work.
/// - Panics if the list of webhooks can't be returned, because the bot
/// won't work.
pub async fn get_link_webhook(
	channel: ChannelId, webhook_name: &str, context: &Context,
) -> Webhook {
	match channel.webhooks(&context).await {
		Ok(w) => match w.iter().find(|i| i.name == Some(webhook_name.to_owned())) {
			Some(w) => w.to_owned(),
			None => match channel.create_webhook(&context, webhook_name).await {
				Ok(w) => w,
				Err(e) => panic!("Error creating webhook: {e}"),
			},
		},
		Err(e) => panic!("Error getting webhooks: {e}"),
	}
}

/// Builds a reply embed using the type provided by [`serenity::model::Message::referenced_message`].
///
/// Should only be used for webhooks.
pub fn build_reply_for_webhook(rm: Box<Message>) -> serde_json::Value {
	Embed::fake(|e| {
		e.description(format!(
			"**[Reply to:]({})** {}{}",
			&rm.link(),
			rm.content
				.chars()
				.take(30)
				.collect::<String>()
				.replace("\n", "")
				.as_mut(),
			if rm.content.len() > 100 {
				"...".to_owned()
			} else {
				"".to_owned()
			}
		))
		.footer(|e| {
			e.icon_url(rm.author.face());
			e.text(rm.author.name)
		})
	})
}

pub async fn propagate_message(
	message: Message, context: &Context, webhooks: &Vec<Webhook>,
) -> (MessageId, Vec<CachedRelated>) {
	let mut sent_webhooks: Vec<CachedRelated> = Vec::new();

	// TODO:
	// This code currently awaits a response from the Discord API for every message
	// sent. Can probably be refactored with a mutex so multiple messages can be
	// sent at once.
	for webhook in webhooks {
		// TODO: Can this be done without taking ownership?t
		let message = message.to_owned();
		let context = context.to_owned();
		let webhook = webhook.to_owned();

		task::spawn(async move {
			if message.channel_id.as_u64() == webhook.channel_id.unwrap().as_u64() {
				return None;
			}

			let res = webhook.execute(&context, true, |wh| {
				wh.content(message.content);
				wh.avatar_url(message.author.face());
				wh.username(message.author.name);
				wh.allowed_mentions(|am| am.parse(ParseValue::Users));
				wh.add_files(message.attachments.iter().fold(
					Vec::<AttachmentType>::new(),
					|mut files, f| {
						files.push(AttachmentType::from(f.url.deref()));

						files
					},
				));

				// Add an embed for replies
				message
					.referenced_message
					.and_then(|rm| Some(wh.embeds(vec![build_reply_for_webhook(rm)])));

				wh
			});

			match res.await {
				Ok(msg) => Some(msg.unwrap()),
				Err(e) => {
					println!("Error sending Discord message {e}");
					None
				}
			}
		})
		.await
		.unwrap()
		.and_then(|v| {
			Some(sent_webhooks.push(CachedRelated {
				related_channel_id: v.channel_id,
				related_message_id: v.id,
				related_webhook_id: v.webhook_id.expect("Wasn't a webhook..."),
			}))
		});
	}

	(message.id, sent_webhooks)
}
