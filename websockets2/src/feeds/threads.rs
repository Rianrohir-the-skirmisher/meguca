use super::{ClientDescriptor, IndexFeed};
use crate::{message::Msg, registry::Registry};
use actix::prelude::*;
use protocol::{payloads::FeedData, Encoder, MessageType};
use rayon::prelude::*;
use serde::Serialize;
use std::{
	collections::{HashMap, HashSet},
	sync::Arc,
};

/// Holds the IDs of up to the last 5 posts
type Last5Posts =
	heapless::BinaryHeap<u64, heapless::consts::U5, heapless::binary_heap::Min>;

/// Update feed. Either a thread feed or the global thread index feed.
#[derive(Debug)]
pub struct Feed {
	/// Thread ID
	id: u64,

	/// Clients needing an init message sent
	need_init: HashSet<u64>,

	/// Cached encoded initialization message buffer
	init_msg_cache: Option<Msg>,

	/// Pending message streaming encoder
	pending: Option<Encoder>,

	/// Cached encoded initialization message buffer for the global thread index
	global_init_msg_cache: Option<Msg>,

	/// Pending messages for global thread index feed
	global_pending: Option<Encoder>,

	/// Last 5 post IDs in thread
	last_5_posts: Last5Posts,

	/// Current active feed data.
	data: FeedData,

	/// Open bodies pending parsing and diffing
	pending_open_bodies: HashMap<u64, String>,

	/// Link to the global registry
	registry: Addr<Registry>,

	/// Link to the thread index Feed
	index_feed: Addr<IndexFeed>,

	/// Clients subscribed to the feed
	clients: HashMap<u64, ClientDescriptor>,
}

impl Actor for Feed {
	type Context = Context<Self>;
}

impl Feed {
	/// Create new wrapped Feed initialized with data
	pub fn new(
		data: FeedData,
		registry: Addr<Registry>,
		index_feed: Addr<IndexFeed>,
	) -> Self {
		// Find last 5 posts added to thread
		let mut l5 = Last5Posts::default();
		for id in data.recent_posts.keys() {
			if match l5.peek() {
				Some(min) => {
					if min < id {
						l5.pop();
						true
					} else {
						false
					}
				}
				None => true,
			} {
				unsafe { l5.push_unchecked(*id) };
			}
		}

		Self {
			id: data.thread,
			registry,
			index_feed,
			last_5_posts: l5,
			data: data,
			need_init: Default::default(),
			init_msg_cache: Default::default(),
			pending: Default::default(),
			global_init_msg_cache: Default::default(),
			global_pending: Default::default(),
			pending_open_bodies: Default::default(),
			clients: Default::default(),
		}
	}

	/// Clear all cached values
	fn clear_cache(&mut self) {
		self.global_init_msg_cache = None;
		self.init_msg_cache = None;
	}

	/// This should never happen, but log it and halt execution, if it does.
	/// Caller should abort execution.
	fn log_encode_error(&self, err: std::io::Error) {
		log::error!(
			"could not encode feed data: feed_id={} err={:?}",
			self.id,
			err
		);
	}

	/// Unwrap or init new Encoder and return it
	fn get_encoder(enc: &mut Option<Encoder>) -> &mut Encoder {
		match enc {
			Some(e) => e,
			None => {
				*enc = Some(Encoder::new(vec![]));
				enc.as_mut().unwrap()
			}
		}
	}

	/// Unwrap or init new Encoder for pending messages and return it
	fn get_pending_encoder(&mut self) -> &mut Encoder {
		Self::get_encoder(&mut self.pending)
	}

	/// Encode and cache feed init message or return cached one.
	fn get_init_msg(&mut self) -> std::io::Result<Msg> {
		match &mut self.init_msg_cache {
			Some(msg) => Ok(msg.clone()),
			None => {
				let msg = Msg::new({
					let mut enc = Encoder::new(Vec::new());
					enc.write_message(MessageType::FeedInit, &self.data)?;
					enc.finish()?
				});
				self.init_msg_cache = Some(msg.clone());
				Ok(msg)
			}
		}
	}

	/// Return, if post should be included in global thread index
	fn include_in_global(&self, id: u64) -> bool {
		id == self.data.thread || self.last_5_posts.iter().any(|x| id == *x)
	}

	/// Encode and cache global feed init message part or return cached one.
	fn get_global_init_msg_part(&mut self) -> std::io::Result<Msg> {
		match &mut self.global_init_msg_cache {
			Some(msg) => Ok(msg.clone()),
			None => {
				let msg = Msg::new({
					let mut enc = Encoder::new(Vec::new());

					macro_rules! filter_recent {
						($key:ident) => {
							self.data
								.$key
								.iter()
								.filter(|(id, _)| self.include_in_global(**id))
								.map(|(k, v)| (*k, v.clone()))
								.collect()
						};
					}
					let (recent_posts, open_posts) = rayon::join(
						|| filter_recent!(recent_posts),
						|| filter_recent!(open_posts),
					);

					enc.write_message(
						MessageType::FeedInit,
						&FeedData {
							thread: self.id,
							recent_posts,
							open_posts,
						},
					)?;
					enc.finish()?
				});
				self.global_init_msg_cache = Some(msg.clone());
				Ok(msg)
			}
		}
	}

	/// Insert new blank open post into the registry
	fn insert_post(&mut self, id: u64, time: u32) {
		self.data.recent_posts.insert(id, time);
		if self.last_5_posts.len() == 5 {
			unsafe { self.last_5_posts.pop_unchecked() };
		}
		unsafe { self.last_5_posts.push_unchecked(id) };

		self.data
			.open_posts
			.insert(id, protocol::payloads::OpenPost::new(self.id, time));
	}

	/// Write post-related message to thread and possibly global feed
	fn encode_post_message(
		&mut self,
		post: u64,
		typ: MessageType,
		payload: &impl Serialize,
	) {
		if let Err(err) = self.get_pending_encoder().write_message(typ, payload)
		{
			self.log_encode_error(err);
		}
		if self.include_in_global(post) {
			if let Err(err) = Self::get_encoder(&mut self.global_pending)
				.write_message(typ, payload)
			{
				self.log_encode_error(err);
			}
		}
	}

	/// Diff pending open post body changes in parallel and write messages to
	/// encoders
	fn diff_open_bodies(&mut self) {
		use protocol::payloads::post_body::{Node, PatchNode};

		for (id, patch, new) in self
			.pending_open_bodies
			.drain()
			.collect::<Vec<(u64, String)>>()
			.into_par_iter()
			.filter_map(|(id, s)| -> Option<(u64, PatchNode, Node)> {
				use crate::body::{diff, parse};

				let old = match self.data.open_posts.get(&id) {
					Some(p) => &p.body,
					// Post already closed
					None => return None,
				};
				let new = match parse(&s, true) {
					Ok(n) => n,
					Err(e) => {
						log::error!("body parsing error on post {}: {}", id, e);
						return None;
					}
				};
				diff(&old, &new).map(|p| (id, p, new))
			})
			.collect::<Vec<(u64, PatchNode, Node)>>()
		{
			let ptr = Arc::new(new);
			self.data.open_posts.get_mut(&id).unwrap().body = ptr.clone();
			crate::body::persist_open_body(id, ptr);
			self.encode_post_message(id, MessageType::PatchPostBody, &patch);
		}
	}
}
