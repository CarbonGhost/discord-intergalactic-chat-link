use std::ops::Deref;

use serenity::{
	builder::ParseValue,
	model::{
		prelude::{AttachmentType, ChannelId, Embed, Message},
		webhook::Webhook,
	},
	prelude::Context,
};

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
				.replace('\n', "")
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

pub async fn execute_message_for_webhook(
	message: Message, context: &Context, webhook: &Webhook,
) -> Result<Option<Message>, serenity::Error> {
	if message.channel_id.as_u64() == webhook.channel_id.unwrap().as_u64() {
		return Ok(None);
	}

	let x = webhook.execute(&context, true, |wh| {
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
			.map(|rm| wh.embeds(vec![build_reply_for_webhook(rm)]));

		wh
	});

	x.await
}
