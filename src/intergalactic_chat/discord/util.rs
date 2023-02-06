use serenity::{
	model::{prelude::ChannelId, webhook::Webhook},
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
