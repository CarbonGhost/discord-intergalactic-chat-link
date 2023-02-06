# Discord Intergalactic Chat Link

![https://discord.gg/kUp9P4jhWv](https://img.shields.io/discord/1072066425591705660?label=discord)
![](https://img.shields.io/github/v/release/carbonghost/discord-intergalactic-chat-link)
![](https://img.shields.io/github/languages/top/carbonghost/discord-intergalactic-chat-link)
![](https://img.shields.io/github/license/carbonghost/discord-intergalactic-chat-link)

**Please be wary that, at this time, this project is considered to be in an alpha state. Support, stability, and security are not guaranteed.**

This is a Discord bot for creating a linked channel between different servers, allowing users to have conversations with each other even if they don't share any servers. This bot is built to be self-hosted first and as such there is no public invite link you can use, however you can find a small demo on [my Discord server](https://discord.gg/kUp9P4jhWv) should you wish to test it
out.

The bot works by sending Discord messages via the lightweight MQTT format to a broker server, then propagating them to the channels you configure. This approach requires you to host your own MQTT server, for which you can find more info [here](#usage).

## Why?

This bot was originally created for the technical Minecraft community, in order to create a channel for members to chat between friendly servers. However the bot doesn't have any specific features, as such is suited to whatever you'd want to use it for.

## Usage

In order to run this bot properly you must have access to both an [MQTT server](https://mosquitto.org/download/) with a static IP and some way to host the bot. However you obtain these two things are up to you, however be aware that any free MQTT providers are likely fully public and I strongly advise against using them.

You can follow these instructions to setup the bot:

1. Create a new application via the [Discord developer portal](https://discord.com/developers/applications).
2. Add a bot to your application and save the token somewhere safe. Anyone with your token can control your bot, so ensure you don't share this with anyone.
3. Download the appropriate binary from the [releases](https://github.com/CarbonGhost/discord-intergalactic-chat-link/releases) for your host machine OS.
4. Open the `config.toml` file and fill in the configuration options. If you have multiple bots all connecting to the same MQTT broker make sure they all have the same `topic`.
5. Add the IDs of the channels you wish to link, these can only be channels on servers the bot is on and has permissions for. While you can set as many as you wish, due to Discord rate limits you're likely to want to keep it below or around 50.
6. Make sure your MQTT server is online and start the bot.

If you need any help you may ask for it on the [support Discord](https://discord.gg/kUp9P4jhWv).

---

[This project is licensed under MIT](https://github.com/CarbonGhost/discord-intergalactic-chat-link/blob/dev/LICENSE.md)
