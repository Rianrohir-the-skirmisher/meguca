use crate::{
	client::Client,
	db,
	feeds::{Feed, IndexFeed},
	util::DynResult,
};
use actix::prelude::*;
use protocol::{payloads::FeedData, util::SetMap};
use std::collections::HashMap;

// TODO: remove clients with disconnected addresses on send

/// Stores client state and address
#[derive(Debug)]
struct ClientDescriptor {
	/// Specific feed a client is synchronized to.
	/// Is unset (default) - before the first sync message is received.
	feed: Option<u64>,

	/// The internal public key ID the client is registered with
	pub_key: Option<u64>,

	/// Address for communication
	addr: Addr<Client>,
}

/// Keeps state and feed subscription of all clients
#[derive(Debug)]
pub struct Registry {
	/// All currently connected clients
	clients: HashMap<u64, ClientDescriptor>,

	/// Maps feed ID to clients are synced to that feed
	by_feed: SetMap<u64, u64>,

	/// Maps client public key ID to a set of clients using that ID
	by_pub_key: SetMap<u64, u64>,

	/// Thread index feed
	index_feed: Addr<IndexFeed>,

	/// All thread feeds in the system. One per existing thread.
	feeds: HashMap<u64, Addr<Feed>>,
}

impl Actor for Registry {
	type Context = actix::Context<Self>;
}

impl Registry {
	/// Initialize Registry instance by reading feed data from the database
	pub fn new(
		ctx: &mut Context<Self>,
		threads: Vec<u64>,
		feed_data: Vec<FeedData>,
	) -> Self {
		let mut feed_data: HashMap<u64, FeedData> =
			feed_data.into_iter().map(|d| (d.thread, d)).collect();

		let index_feed = IndexFeed::new(ctx.address()).start();

		Self {
			clients: Default::default(),
			by_feed: Default::default(),
			by_pub_key: Default::default(),
			index_feed: index_feed.clone(),
			feeds: threads
				.into_iter()
				.map(move |id| {
					let mut data = feed_data.remove(&id).unwrap_or_default();
					data.thread = id;
					(
						id,
						Feed::new(data, ctx.address(), index_feed.clone())
							.start(),
					)
				})
				.collect(),
		}
	}
}

/// Request to register a client
#[derive(Message)]
#[rtype(result = "()")]
pub struct RegisterClient {
	pub id: u64,
	pub addr: Addr<Client>,
}

impl Handler<RegisterClient> for Registry {
	type Result = ();

	fn handle(
		&mut self,
		msg: RegisterClient,
		_: &mut Self::Context,
	) -> Self::Result {
		self.clients.insert(
			msg.id,
			ClientDescriptor {
				feed: None,
				pub_key: None,
				addr: msg.addr,
			},
		);
	}
}

/// Remove client from registry by ID
#[derive(Message)]
#[rtype(result = "()")]
pub struct UnregisterClient(pub u64);

impl Handler<UnregisterClient> for Registry {
	type Result = ();

	fn handle(
		&mut self,
		UnregisterClient(client): UnregisterClient,
		_: &mut Self::Context,
	) -> Self::Result {
		if let Some(desc) = self.clients.remove(&client) {
			if let Some(feed) = desc.feed {
				self.by_feed.remove(&feed, &client);
			}
			if let Some(pub_key) = desc.pub_key {
				self.by_pub_key.remove(&pub_key, &client);
			}
		}
	}
}

/// Set client feed
#[derive(Message)]
#[rtype(result = "Result<AnyFeed, String>")]
pub struct SetFeed {
	pub client: u64,
	pub feed: u64,
}

/// Either a thread or thread index feed
pub enum AnyFeed {
	Index(IndexFeed),
	Thread(Feed),
}

impl Handler<SetFeed> for Registry {
	type Result = Result<AnyFeed, String>;

	fn handle(&mut self, msg: SetFeed, _: &mut Self::Context) -> Self::Result {
		if let Some(desc) = self.clients.get_mut(&msg.client) {
			if match desc.feed.take() {
				Some(old_feed) => {
					if old_feed != msg.feed {
						self.by_feed.remove(&old_feed, &msg.client);
						true
					} else {
						false
					}
				}
				None => true,
			} {
				self.by_feed.insert(msg.feed, msg.client);
				desc.feed = Some(msg.feed);
				// TODO: return error, if no such feed exists
				// TODO: pass client to feed
			}
		}
		Ok()
	}
}

/// Set client public key
#[derive(Message)]
#[rtype(result = "()")]
pub struct SetPublicKey {
	pub client: u64,
	pub pub_key: u64,
}

impl Handler<SetPublicKey> for Registry {
	type Result = ();

	fn handle(
		&mut self,
		SetPublicKey { client, pub_key }: SetPublicKey,
		_: &mut Self::Context,
	) -> Self::Result {
		if let Some(desc) = self.clients.get_mut(&client) {
			self.by_pub_key.insert(pub_key, client);
			desc.pub_key = Some(pub_key);
		}
	}
}
