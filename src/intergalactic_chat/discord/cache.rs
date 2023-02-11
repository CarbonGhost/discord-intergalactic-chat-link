use std::{
	collections::HashMap,
	fs::OpenOptions,
	io::{self, Read, Write},
	ops::Deref,
};

use serde::{Deserialize, Serialize};
use serenity::model::prelude::{ChannelId, MessageId, WebhookId};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MessageCache {
	/// The maximum number of entires the cache can hold before removing old ones.
	size: usize,
	/// A hash map representing the cached values.
	///
	/// The key represents the original message sent by the user and the value
	/// contains data related to messages the bot has posted.
	cache: HashMap<MessageId, Vec<CacheValue>>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CacheValue {
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

	/// Pushes the value into the cache and removes one value if the cache is too
	/// large.
	pub fn push(&mut self, k: MessageId, v: Vec<CacheValue>) -> &mut Self {
		self.pop_if_oversized();
		self.cache.insert(k, v);

		self
	}

	/// Pushes into the cache value by `k`.
	pub fn push_into_value(&mut self, k: MessageId, v: CacheValue) -> &mut Self {
		self.pop_if_oversized();
		self.cache.entry(k).and_modify(|e| e.push(v));

		self
	}

	pub fn initialize(path: &str, size: usize) -> Self {
		let mut buf = String::new();

		OpenOptions::new()
			.read(true)
			.write(true)
			.create(true)
			.open(path)
			.unwrap()
			.read_to_string(&mut buf)
			.unwrap();

		if buf.len() == 0 {
			Self::new(size)
		} else {
			serde_json::from_str::<Self>(buf.deref()).unwrap()
		}
	}

	/// Writes [`MessageCache`] to the file provided by `path`. If that file does
	/// not exist, a new one will be created.
	pub fn write_to_file(self, path: &str) -> Result<Self, io::Error> {
		OpenOptions::new()
			.read(true)
			.write(true)
			.create(true)
			.truncate(true)
			.open(path)
			.unwrap()
			.write_all(serde_json::to_string(&self).unwrap().as_bytes())
			.unwrap();

		Ok(self)
	}

	pub fn get_entry(&self, k: &MessageId) -> Option<(&MessageId, &Vec<CacheValue>)> {
		self.cache.get_key_value(k)
	}

	pub fn remove(&mut self, k: &MessageId) -> &mut Self {
		self.cache.remove(k);

		self
	}

	/// Checks that the cache is not larger than than `size`. If it is larger than
	/// `size` than the adequate number of items are removed as to make the cache
	/// exactly equal to `size`.
	fn pop_if_oversized(&mut self) -> &mut Self {
		// If the difference is greater than zero, the cache is too big
		let z: i32 = self.size.try_into().unwrap();
		let l: i32 = self.cache.len().try_into().unwrap();
		let diff: i32 = l - z;

		if diff > 0 {
			let keys: Vec<_> = {
				self.cache
					.keys()
					.take(diff.unsigned_abs().try_into().unwrap())
					.copied()
					.collect()
			};
			for k in keys {
				self.cache.remove(&k);
			}
		}

		self
	}
}
