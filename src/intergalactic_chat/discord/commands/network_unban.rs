use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
	ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
};
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::model::Permissions;
use serenity::prelude::Context;

use crate::intergalactic_chat::discord::bot::DiscordHandler;

pub async fn run(
	options: &[CommandDataOption], command: &ApplicationCommandInteraction, context: &Context,
	handler: &DiscordHandler,
) -> Result<(), serenity::Error> {
	let user_option = options
		.get(0)
		.expect("Expected user option")
		.resolved
		.as_ref()
		.expect("Expected user object");

	let content = if let CommandDataOptionValue::User(user, _) = user_option {
		// If the user ID is not in the ban list:
		if handler.ban_list.lock().await.list.get(&user.id).is_none() {
			"This user is not banned. Are you looking for the `/network-ban` command?".to_owned()
		} else {
			handler.ban_list.lock().await.list.remove(&user.id);

			let was_notified_message = match user.dm(&context, |dm| {
				dm.content("You have been unbanned from the network by a moderator. Your messages can now be sent to other servers.".to_owned())
			}).await {
				Ok(_) => "successfully notified",
				Err(_) => "unable to be notified"
			};

			format!(
				"Unbanned <@{}> from the chat link.\n\nThe user was {} by direct message.",
				user.id, was_notified_message
			)
		}
	} else {
		"The user provided does not exist.".to_owned()
	};

	command
		.create_interaction_response(&context.http, |r| {
			r.kind(InteractionResponseType::ChannelMessageWithSource);
			r.interaction_response_data(|rd| {
				rd.content(content);
				rd.ephemeral(true)
			})
		})
		.await
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	command
		.name("network-unban")
		.description("Removes a ban from a user if they have one.")
		.default_member_permissions(Permissions::BAN_MEMBERS)
		.create_option(|option| {
			option
				.name("user")
				.description("The user to unban.")
				.kind(CommandOptionType::User)
				.required(true)
		})
}
