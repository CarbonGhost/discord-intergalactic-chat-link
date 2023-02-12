use serenity::builder::CreateApplicationCommand;
use serenity::model::prelude::interaction::application_command::{
	ApplicationCommandInteraction, CommandDataOption,
};
use serenity::model::prelude::interaction::InteractionResponseType;
use serenity::prelude::Context;

pub async fn run(
	_options: &[CommandDataOption], command: &ApplicationCommandInteraction, context: &Context,
) -> Result<(), serenity::Error> {
	command
		.create_interaction_response(&context.http, |r| {
			r.kind(InteractionResponseType::ChannelMessageWithSource);
			r.interaction_response_data(|rd| {
				rd.content("Intergalactic Chat Link is an open source Discord bot created by CarbonGhost. Find out more about the bot, contribute, or report an issue on [GitHub](https://github.com/CarbonGhost/discord-intergalactic-chat-link).");
				rd.ephemeral(true)
			})
		})
		.await
}

pub fn register(command: &mut CreateApplicationCommand) -> &mut CreateApplicationCommand {
	command
		.name("about")
		.description("Returns information about the bot.")
}
