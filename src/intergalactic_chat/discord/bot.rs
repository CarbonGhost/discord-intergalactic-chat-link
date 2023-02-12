use std::ops::Deref;
use std::process::exit;
use std::str::from_utf8;
use std::sync::Arc;
use std::time::Instant;

use crate::intergalactic_chat::discord::commands;
use crate::intergalactic_chat::discord::util::{execute_message_for_webhook, get_link_webhook};
use crate::Config;
use rumqttc::{AsyncClient, Event, Incoming, QoS};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::id::ChannelId;
use serenity::model::prelude::interaction::Interaction;
use serenity::model::prelude::{GuildId, MessageId, MessageUpdateEvent};
use serenity::model::webhook::Webhook;
use serenity::prelude::*;
use tokio::sync::broadcast;
use tokio::{signal, task};

use super::bans::BanList;
use super::cache::{CacheValue, MessageCache};

pub struct DiscordHandler {
	pub mq_client: AsyncClient,
	pub mq_event_receiver: broadcast::Receiver<Event>,
	pub config: Config,
	pub message_cache: Arc<Mutex<MessageCache>>,
	pub ban_list: Arc<Mutex<BanList>>,
}

#[async_trait]
impl EventHandler for DiscordHandler {
	async fn ready(&self, context: Context, ready: Ready) {
		println!("Setting things up...");

		let reg_wh_start = Instant::now();
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

		println!(
			"Setup {} webhooks in {:#?}...",
			&webhooks.len(),
			reg_wh_start.elapsed()
		);

		let reg_cmd_start = Instant::now();
		let mut command_names: Vec<String> = Vec::new();

		for guild in &ready.guilds {
			let guild_id = guild.id;

			let commands =
				GuildId::set_application_commands(&guild_id, &context.http, |commands| {
					commands
						.create_application_command(|command| commands::ping::register(command))
						.create_application_command(|command| commands::about::register(command))
						.create_application_command(|command| {
							commands::network_ban::register(command)
						})
						.create_application_command(|command| {
							commands::network_unban::register(command)
						})
				})
				.await
				.unwrap();

			for command in commands {
				command_names.push(command.name);
			}
		}

		println!(
			"Setup {:#?} commands in {:#?}...\n",
			command_names.len(),
			reg_cmd_start.elapsed()
		);

		println!(
			"{} connected to Discord and ready to start receiving events!",
			ready.user.name
		);
		println!("Invite with: https://discord.com/api/oauth2/authorize?client_id={}&permissions=1789592463424&scope=bot", ready.application.id);
		println!(
			"Watching {} channels in {} servers\n",
			&self.config.discord.channels.len(),
			ready.guilds.len()
		);

		let message_cache = Arc::clone(&self.message_cache);
		let ban_list = Arc::clone(&self.ban_list);

		task::spawn(async move {
			loop {
				match signal::ctrl_c().await {
					Ok(()) => {
						Arc::clone(&message_cache)
							.lock()
							.await
							.to_owned()
							.write_to_file(".cache")
							.unwrap();

						Arc::clone(&ban_list)
							.lock()
							.await
							.to_owned()
							.write_to_file(".bans")
							.unwrap();

						println!("Goodbye!");

						exit(0)
					}
					Err(e) => {
						eprintln!("Unable to listen for shutdown signal: {e}");
					}
				}
			}
		});

		// Process the event and ensure it's a valid Discord message. The loop
		// will simply return if a problem is found, as none of the issues are
		// unrecoverable.
		// TODO: This can definitely be done more efficiently!
		loop {
			let message = match event_receiver.recv().await {
				Ok(Event::Incoming(Incoming::Publish(p))) => match from_utf8(&p.payload) {
					Ok(p) => match serde_json::from_str::<Message>(p) {
						Ok(p) => p,
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
								message_cache.lock().await.push_into_value(
									message_id,
									CacheValue {
										related_channel_id: m.channel_id,
										related_message_id: m.id,
										related_webhook_id: webhook.id,
									},
								);
							}
							None => (),
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
		let c = &mut self.message_cache.lock().await;
		let cache_value = c.get_entry(&deleted_message_id);

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

		c.remove(&deleted_message_id);
	}

	async fn message_update(&self, context: Context, new_data: MessageUpdateEvent) {
		let new_content = match new_data.content {
			Some(v) => v,
			None => return,
		};

		let c = &self.message_cache.lock().await;
		let cache_value = c.get_entry(&new_data.id);

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
		} else if self
			.ban_list
			.lock()
			.await
			.list
			.contains_key(&message.author.id)
		{
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

	async fn interaction_create(&self, context: Context, interaction: Interaction) {
		if let Interaction::ApplicationCommand(command) = interaction {
			match command.data.name.as_str() {
				"ping" => commands::ping::run(&command.data.options, &command, &context)
					.await
					.unwrap(),
				"about" => commands::about::run(&command.data.options, &command, &context)
					.await
					.unwrap(),
				"network-ban" => {
					commands::network_ban::run(&command.data.options, &command, &context, &self)
						.await
						.unwrap()
				}
				"network-unban" => {
					commands::network_unban::run(&command.data.options, &command, &context, &self)
						.await
						.unwrap()
				}
				_ => panic!("TODO: Unhandled command"),
			};
		}
	}
}
