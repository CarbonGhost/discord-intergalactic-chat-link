use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::command::CommandOptionType;
use serenity::model::prelude::interaction::application_command::{
	ApplicationCommandInteraction, CommandDataOption, CommandDataOptionValue,
};
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::model::Permissions;
use serenity::prelude::Context;

use crate::intergalactic_chat::discord::bans::BanEntry;
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
	let reason_option = options
		.get(1)
		.expect("Expected `reason` option")
		.resolved
		.as_ref()
		.expect("Expected string");

	let content = if let CommandDataOptionValue::User(user, _) = user_option {
		// If the user ID is already in the ban list:
		if handler.ban_list.lock().await.list.get(&user.id).is_some() {
			"This user has already been banned. Are you looking for the `/network-unban` command?"
				.to_owned()
		} else {
			user.bot.then(|| "You cannot network ban bot users, if you wish to achieve the same result, try updating their permissions"
			.to_owned());

			let reason = if let CommandDataOptionValue::String(reason) = reason_option {
				reason.to_owned()
			} else {
				"Invalid reason".to_owned()
			};

			handler.ban_list.lock().await.list.insert(
				user.id,
				// These unwraps are safe because the command is only registered in guilds
				// and will never be `None`.
				BanEntry {
					reason: reason.to_owned(),
					executor: command.member.to_owned().unwrap().user.id,
					ban_origin: command.guild_id.unwrap(),
					timestamp: command.id.created_at(),
				},
			);

			let was_notified_message = match user.dm(&context, |dm| {
				dm.content(format!("You have been network banned by a moderator. This prevents your messages from being sent to other servers, but you can still read and sent messages in linked channels.\n\nThe moderators have provided a reason for your ban:\n\"{}\"", reason))
			}).await {
				Ok(_) => "successfully notified",
				Err(_) => "unable to be notified"
			};

			format!(
				"Banned <@{}> from the chat link with the reason: \"{}\"\n\nThe user was {} by direct message.",
				user.id,
				reason.to_owned(),
				was_notified_message
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
		.name("network-ban")
		.description("Prevents a users messages from being propagated.")
		.default_member_permissions(Permissions::BAN_MEMBERS)
		.create_option(|option| {
			option
				.name("user")
				.description("The user to ban.")
				.kind(CommandOptionType::User)
				.required(true)
		})
		.create_option(|option| {
			option
				.name("reason")
				.description("Why was this user banned?")
				.kind(CommandOptionType::String)
				.required(true)
		})
}
