use serenity::{
	model::{
		prelude::{ChannelId, Embed, Message},
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
