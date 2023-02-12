![Discord Intergalactic Chat Link](https://user-images.githubusercontent.com/39361743/217791136-a997860e-4db3-4150-9a7f-07084ac07c3b.jpg)

![](https://img.shields.io/github/v/release/carbonghost/discord-intergalactic-chat-link)
![](https://img.shields.io/github/actions/workflow/status/carbonghost/discord-intergalactic-chat-link/main.yml)
![](https://img.shields.io/github/languages/top/carbonghost/discord-intergalactic-chat-link)
![](https://img.shields.io/github/license/carbonghost/discord-intergalactic-chat-link)
![https://discord.gg/kUp9P4jhWv](https://img.shields.io/discord/1072066425591705660?color=5666f2)

> **Please be wary that, at this time, this project is considered to be in an alpha state. Support, stability, and security are not guaranteed.**

This is a Discord bot for creating linked channels between different servers, allowing users to have conversations with each other even if they don't share any servers. This bot is built to be self-hosted first and as such there is no public invite link you can use, however you can find a small demo on [my Discord server](https://discord.gg/kUp9P4jhWv) should you wish to test it out.

The bot works by sending Discord messages via the lightweight MQTT format to a broker server, then propagating them to the channels you configure. This approach requires you to host your own MQTT server, for which you can find more info [here](#usage).

## Features

- Have conversations even if you don't share servers.
- Handle multiple servers and channels with one bot.
- Support for attachments and replies.
- Ban users from the network.
- Support for edits and deletions.

## Why?

This bot was originally created for the technical Minecraft community, in order to create a channel for members to chat between friendly servers. However the bot doesn't have any specific features, as such is suited to whatever you'd want to use it for.

## Usage

In order to run this bot properly you must have access to both an [MQTT server](https://mosquitto.org/download/) with a static IP and some way to host the bot. However you obtain these two things are up to you, however be aware that any free MQTT providers are likely fully public and I strongly advise against using them.

You can follow these instructions to setup the bot:

1. Create a new application via the [Discord developer portal](https://discord.com/developers/applications).
2. Add a bot to your application and save the token somewhere safe. Anyone with your token can control your bot, so ensure you don't share this with anyone.
3. Download the appropriate binary from the [releases](https://github.com/CarbonGhost/discord-intergalactic-chat-link/releases) for your host machine OS.
4. Open the `config.toml` file and fill in the configuration options. If you have multiple bots all connecting to the same MQTT broker make sure they all have the same `topic`.
5. Add the IDs of the channels you wish to link, these can only be channels on servers the bot is on and has permissions for.
6. Make sure your MQTT server is online and start the bot.

If you need any help you may ask for it on the [support Discord](https://discord.gg/kUp9P4jhWv).

<details><summary>Default config</summary>
<p>

```toml
# This is the configuration file for your bot, make sure it is valid
# before starting the bot.

# For help and more information about the bot go 
# to: https://github.com/CarbonGhost/discord-intergalactic-chat-link

[mqtt]
broker_ip = "localhost" # The IP address of your broker server.
broker_port = 1883 # The port the server is using, by default "1883".
client_id = "bot" # The client ID used to connect to the MQTT server.
topic = "example/topic" # The topic you wish to send / receive messages through.

[discord]
bot_id = 0000000000000000000 # The application ID of your bot, found via the Discord Developer Portal.
# A list of channels IDs for channels you wish for the bot to link, 
# separated by commas.
# You can have any number of channels on any number of servers, but the
# bot must have access to them and be able to create a webhook.
channels = [
	0000000000000000000,
	0000000000000000000,
	0000000000000000000,
]
# The bot's token, found via the Discord Developer portal.
# If you are reporting an issue make sure to omit this value!
token = "XXXXXXXXXXXXXXXXXXXXXXXXXX.XXXXXX.XXXXXXXX-XXXXXXXXXXXXXXXXXXXXXXXXXXXXX"
```

</p>
</details>

## Development & Building From Source

If you wish to contribute to the project or just setup the bot from source you can do so simply by cloning the repository and running:

```bash
$ cargo run
```

...to run the bot in a development environment, or:

```bash
$ cargo build --release
```

...to build the bot for production.

If you're submitting a pull request I ask that you use [rustfmt](https://github.com/rust-lang/rustfmt) to format your code appropriately.

---

This project is open source under the [MIT license](https://github.com/CarbonGhost/discord-intergalactic-chat-link/blob/dev/LICENSE.md). The logo is generated by AI, any representation of real art is unintentional.
