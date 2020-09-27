mod index;
mod threads;

pub use index::*;
pub use threads::*;

use crate::client::Client;
use actix::prelude::*;

/// Stores client state and address
#[derive(Debug)]
struct ClientDescriptor {
	/// Client requires init message
	need_init: bool,

	/// Address for communication
	addr: Addr<Client>,
}

impl ClientDescriptor {
	fn new(addr: Addr<Client>) -> Self {
		Self {
			addr,
			need_init: true,
		}
	}
}

// TODO: use old Feed architecture from v7, except with a global feed that also
// receives all messages concerning the OP and last 5 posts
// TODO: keep track of clients that need init on the Feed itself
// TODO: merge FeedCommon and Feed as the global feed Actor will be very
// different from thread feed actors
// TODO: separate mutable and immutable pages on Feed. Store immutable pages in
// immutable memory mapped files.
// TODO: keep feed address on the client itself

// pub fn init(feed_data: &[u8]) -> DynResult {
// 	let (sdr, recv) = channel();
// 	unsafe {
// 		REQUEST = Some(Mutex::new(sdr));
// 	}
// 	let mut p: Pulsar = Default::default();
// 	p.init(feed_data)?;
// 	std::thread::Builder::new()
// 		.name("pulsar".into())
// 		.spawn(move || {
// 			const SEND_INTERVAL: Duration = Duration::from_millis(100);
// 			const CLEANUP_INTERVAL: Duration = Duration::from_secs(60);

// 			let now = Instant::now();
// 			let mut last_send = now;
// 			let mut last_cleanup = now;

// 			loop {
// 				let started = Instant::now();

// 				// Process all pending requests
// 				for req in recv.try_iter() {
// 					use Request::*;

// 					match req {
// 						InsertThread(data) => p.insert_thread(data),
// 						InsertPost(data) => p.insert_post(data),
// 						RemoveThread(id) => p.remove_thread(id),
// 						InsertImage(req) => p.insert_image(req),
// 						SetOpenBody { post, thread, body } => {
// 							p.enqueue_open_body(post, thread, body)
// 						}
// 					}
// 				}

// 				if started - last_send > SEND_INTERVAL {
// 					last_send = now;

// 					// Block until messages are sent to the Go side to guarantee
// 					// sequentiality
// 					p.send_messages();
// 				}
// 				if started - last_cleanup > CLEANUP_INTERVAL {
// 					last_cleanup = now;
// 					p.clean_up();
// 				}

// 				// Sleep thread to save resources.
// 				// Compensate for a possibly long tick.
// 				let elapsed = Instant::now() - started;
// 				let mut dur = Duration::from_millis(30);
// 				// Duration can not be negative
// 				if elapsed < dur {
// 					dur -= elapsed;
// 				}
// 				if dur.as_millis() != 0 {
// 					std::thread::sleep(dur);
// 				}
// 			}
// 		})?;
// 	Ok(())
// }

// impl Pulsar {
// 	/// Initialize pulsar instance by reading feed data from the database
// 	pub async fn new(registry: Addr<Registry>) -> DynResult<Self> {
// 		Ok(Pulsar {
// 			feeds: db::get_feed_data()
// 				.await?
// 				.into_iter()
// 				.map(|d| (d.thread, Feed::new(d)))
// 				.collect(),
// 			global: Default::default(),
// 			registry,
// 		})
// 	}
// }

// impl Pulsar {

// 	/// Register a new thread and allocate its resources
// 	fn insert_thread(&mut self, data: ThreadCreationNotice) {
// 		self.global.clear_cache();

// 		let mut f = Feed::new(FeedData {
// 			thread: data.id,
// 			recent_posts: Default::default(),
// 			open_posts: Default::default(),
// 		});
// 		f.insert_post(data.id, data.time);
// 		self.feeds.insert(data.id, f);

// 		if let Err(e) = self
// 			.global
// 			.get_pending_encoder()
// 			.write_message(MessageType::InsertThread, &data)
// 		{
// 			self.global.log_encode_error(e);
// 		}
// 	}

// 	/// Register a new post and allocate its resources
// 	fn insert_post(&mut self, data: PostCreationNotification) {
// 		self.mod_thread(data.thread, |f| {
// 			f.insert_post(data.id, data.time);
// 			f.encode_post_message(data.id, MessageType::InsertPost, &data);
// 		});
// 	}

// 	/// Deallocate thread resources and redirect all of its clients
// 	fn remove_thread(&mut self, _id: u64) {
// 		self.global.clear_cache();

// 		todo!(concat!(
// 			"Remove feed data, redirect clients on thread deletion, ",
// 			"clear cache, pass message to global feed"
// 		))
// 	}

// 	/// Log an item has not been found
// 	fn log_not_found(label: &str, id: impl std::fmt::Debug) {
// 		bindings::log_error(&format!(
// 			"{} not found: {:?}\n{:?}",
// 			label,
// 			id,
// 			backtrace::Backtrace::new()
// 		))
// 	}

// 	fn mod_thread(&mut self, thread: u64, handler: impl FnOnce(&mut Feed)) {
// 		match self.feeds.get_mut(&thread) {
// 			Some(f) => {
// 				self.global.clear_cache();
// 				f.clear_cache();
// 				handler(f);
// 			}
// 			None => Self::log_not_found("thread", thread),
// 		}
// 	}

// 	/// Insert an image into an allocated post
// 	fn insert_image(&mut self, req: ImageInsertionReq) {
// 		self.mod_thread(req.thread, |f| {
// 			match f.data.open_posts.get_mut(&req.post) {
// 				Some(p) => {
// 					p.has_image = true;
// 					p.image_spoilered = req.img.spoilered;
// 					f.encode_post_message(
// 						req.post,
// 						MessageType::InsertImage,
// 						&protocol::payloads::InsertImage {
// 							post: req.post,
// 							image: req.img.clone(),
// 						},
// 					);
// 				}
// 				None => {
// 					Self::log_not_found("open post", (req.thread, req.post))
// 				}
// 			}
// 		})
// 	}

// 	/// Enqueue open body for parsing and diffing on next pulse
// 	fn enqueue_open_body(&mut self, post: u64, thread: u64, body: String) {
// 		self.mod_thread(thread, |f| {
// 			f.pending_open_bodies.insert(post, body);
// 		});
// 	}

// 	/// Clean up expired recent posts
// 	fn clean_up(&mut self) {
// 		let threshold = (SystemTime::now() - Duration::from_secs(60 * 15))
// 			.elapsed()
// 			.unwrap_or(Duration::from_secs(0))
// 			.as_secs();
// 		self.feeds.par_iter_mut().for_each(|(_, feed)| {
// 			feed.data
// 				.recent_posts
// 				.retain(|_, created_on| *created_on > threshold as u32)
// 		})
// 	}

// 	/// Generate, aggregate and send buffered messages to clients
// 	fn send_messages(&mut self) {
// 		// TODO: Make client filter recent posts by creation timestamp to the
// 		// last 15 min

// 		// Need a partial snapshot of the registry for atomicity
// 		let clients_by_feed = registry::snapshot_threads(|sm| {
// 			let mut not_ready = Vec::<(u64, HashSet<u64>)>::new();
// 			for (feed, clients) in sm.drain() {
// 				if feed == 0 {
// 					self.global.need_init.extend(clients);
// 					continue;
// 				}
// 				match self.feeds.get_mut(&feed) {
// 					Some(f) => f.common.need_init.extend(clients),
// 					None => not_ready.push((feed, clients)),
// 				};
// 			}
// 			if not_ready.len() != 0 {
// 				*sm = not_ready.into_iter().collect();
// 			}
// 		});

// 		let messages = self.aggregate_feed_messages(&clients_by_feed);

// 		let mut messages_by_client = HashMap::new();

// 		// Assign thread feed messages to all thread feed clients
// 		for thread_message in messages.thread_messages {
// 			messages_by_client
// 				.insert(thread_message.client, thread_message.msg);
// 		}

// 		self.assign_global_feed_messages(
// 			messages.global_feed_messages,
// 			clients_by_feed.get(&0),
// 			&mut messages_by_client,
// 		);
// 		self.merge_server_wide_messages(
// 			&clients_by_feed,
// 			&mut messages_by_client,
// 		);

// 		#[cfg(debug_assertions)]
// 		if !messages_by_client.is_empty() {
// 			debug_log!("messages by client", messages_by_client);
// 		}

// 		// Send all messages in parallel to maximize parallelism of the Go side
// 		messages_by_client
// 			.into_par_iter()
// 			.for_each(|(client, msg)| {
// 				bindings::write_message(client, msg.into());
// 			})
// 	}

// 	/// Aggregate feed messages to send for all thread feeds and the global feed
// 	fn aggregate_feed_messages(
// 		&mut self,
// 		clients_by_feed: &SetMap<u64, u64>,
// 	) -> MessageSet {
// 		self.feeds
// 			.par_iter_mut()
// 			.filter_map(|(id, f)| -> Option<MessageSet> {
// 				if f.common.need_init.is_empty()
// 					&& clients_by_feed.get(id).is_none()
// 				{
// 					return None;
// 				}

// 				// Compute splice messages from stored post body string pairs
// 				// first as those can append to pending message encoders
// 				f.diff_open_bodies();

// 				#[rustfmt::skip]
// 				macro_rules! handle_error {
// 					($res:expr) => {
// 						match $res {
// 							Ok(v) => v,
// 							Err(err) => {
// 								f.log_encode_error(err);
// 								return None;
// 							}
// 						}
// 					};
// 				}

// 				#[rustfmt::skip]
// 				macro_rules! make_msg {
// 					($res:expr) => {{
// 						Msg::new(handle_error!($res))
// 					}};
// 				}

// 				let thread_messages: Vec<ClientMessage> = match (
// 					!f.common.need_init.is_empty(),
// 					f.common.pending.take(),
// 					clients_by_feed.get(id),
// 				) {
// 					(true, None, Some(clients)) => {
// 						let msg = handle_error!(f.get_init_msg());
// 						f.common
// 							.need_init
// 							.drain()
// 							.filter(|c| clients.contains(&c))
// 							.map(|c| ClientMessage {
// 								client: c,
// 								msg: msg.clone(),
// 							})
// 							.collect()
// 					}
// 					(true, Some(pending), Some(clients)) => {
// 						let single = make_msg!(pending.finish());
// 						// Init messages should be sent first to maintain
// 						// event sequentiality
// 						let with_init = Msg::new(Encoder::join(&[
// 							&handle_error!(f.get_init_msg()),
// 							&single,
// 						]));

// 						clients
// 							.iter()
// 							.map(|c| ClientMessage {
// 								client: *c,
// 								msg: if f.common.need_init.contains(c) {
// 									with_init.clone()
// 								} else {
// 									single.clone()
// 								},
// 							})
// 							.collect()
// 					}
// 					(false, Some(pending), Some(clients)) => {
// 						let msg = make_msg!(pending.finish());
// 						clients
// 							.iter()
// 							.cloned()
// 							.map(|c| ClientMessage {
// 								client: c,
// 								msg: msg.clone(),
// 							})
// 							.collect()
// 					}
// 					// If no clients, simply drop the full encoder
// 					_ => Default::default(),
// 				};
// 				// Always clear clients needing init, as they were either
// 				// handled above or ignored due to having navigated away or
// 				// disconnected
// 				f.common.need_init.clear();

// 				Some(MessageSet {
// 					thread_messages: thread_messages,
// 					global_feed_messages: match f.global_pending.take() {
// 						Some(pending) => vec![make_msg!(pending.finish())],
// 						None => Default::default(),
// 					},
// 				})
// 			})
// 			.reduce(
// 				|| Default::default(),
// 				|mut a, mut b| {
// 					#[rustfmt::skip]
// 					macro_rules! merge {
// 						($($key:ident),+) => {
// 							$(
// 								// Extend the bigger collection to reduce
// 								// reallocations
// 								if b.$key.capacity() > a.$key.capacity()  {
// 									std::mem::swap(&mut a.$key, &mut b.$key);
// 								}
// 								a.$key.extend(b.$key);
// 							)+
// 						};
// 					}

// 					merge!(global_feed_messages, thread_messages);
// 					a
// 				},
// 			)
// 	}

// 	/// Add message to all clients either before or after any existing messages
// 	#[inline]
// 	fn add_to_clients(
// 		messages_by_client: &mut HashMap<u64, Msg>,
// 		msg: &Msg,
// 		clients: impl IntoIterator<Item = u64>,
// 		before: bool,
// 	) {
// 		// For clients with messages
// 		messages_by_client.par_iter_mut().for_each(|(_, queued)| {
// 			*queued = if before {
// 				queued.prepend(&msg)
// 			} else {
// 				queued.append(&msg)
// 			};
// 		});
// 		// For clients without messages
// 		for c in clients {
// 			if !messages_by_client.contains_key(&c) {
// 				messages_by_client.insert(c, msg.clone());
// 			}
// 		}
// 	}

// 	/// Assign global feed messages to clients on the global feed
// 	fn assign_global_feed_messages(
// 		&mut self,
// 		global_feed_messages: Vec<Msg>,
// 		global_clients: Option<&HashSet<u64>>,
// 		messages_by_client: &mut HashMap<u64, Msg>,
// 	) {
// 		// Assign init messages to clients needing them
// 		if !self.global.need_init.is_empty() {
// 			println!("need init on global: {:?}", self.global.need_init);
// 			let msg = match &mut self.global.init_msg_cache {
// 				Some(msg) => msg.clone(),
// 				None => {
// 					let msg = Msg::from(Encoder::join(
// 						&self
// 							.feeds
// 							.par_iter_mut()
// 							.filter_map(|(_, f)| {
// 								match f.get_global_init_msg_part() {
// 									Ok(msg) => Some(msg),
// 									Err(e) => {
// 										f.log_encode_error(e);
// 										None
// 									}
// 								}
// 							})
// 							.collect::<Vec<_>>(),
// 					));
// 					self.global.init_msg_cache = msg.clone().into();
// 					msg
// 				}
// 			};

// 			Self::add_to_clients(
// 				messages_by_client,
// 				&msg,
// 				self.global.need_init.drain(),
// 				// Init messages should be processed first to maintain event
// 				// sequentiality
// 				true,
// 			);
// 		}

// 		// Assign global feed messages to clients
// 		if let (true, Some(clients)) =
// 			(!global_feed_messages.is_empty(), global_clients)
// 		{
// 			let msg = Msg::new(Encoder::join(global_feed_messages));
// 			for c in clients.iter().cloned() {
// 				messages_by_client.insert(c, msg.clone());
// 			}
// 		}
// 	}

// 	/// Merge server-wide messages to all clients.
// 	/// Not very efficient, but that is fine. These happen rarely.
// 	fn merge_server_wide_messages(
// 		&mut self,
// 		clients_by_feed: &SetMap<u64, u64>,
// 		messages_by_client: &mut HashMap<u64, Msg>,
// 	) {
// 		if let Some(pending) = self.global.pending.take() {
// 			match pending.finish() {
// 				Ok(buf) => {
// 					Self::add_to_clients(
// 						messages_by_client,
// 						&buf.into(),
// 						clients_by_feed.values().copied(),
// 						false,
// 					);
// 				}
// 				Err(err) => self.global.log_encode_error(err),
// 			};
// 		}
// 	}
// }

// #[derive(Debug)]
// pub struct ImageInsertionReq {
// 	thread: u64,
// 	post: u64,
// 	img: Image,
// }

// /// Request to pulsar
// #[derive(Debug)]
// pub enum Request {
// 	/// Register a freshly-created thread
// 	InsertThread(ThreadCreationNotice),

// 	/// Register a freshly-created post
// 	InsertPost(PostCreationNotification),

// 	/// Deallocate thread resources and redirect all of its clients
// 	RemoveThread(u64),

// 	/// Insert an image into an allocated post
// 	InsertImage(ImageInsertionReq),

// 	/// Set the body of an open post
// 	SetOpenBody {
// 		post: u64,
// 		thread: u64,
// 		body: String,
// 	},
// }

// /// Alias Result for sending a request to Pulsar
// pub type SendResult = Result<(), SendError<Request>>;

// fn send_request(req: Request) -> SendResult {
// 	unsafe { REQUEST.as_ref().unwrap().lock().unwrap().clone() }.send(req)
// }

// /// Initialize a freshly-created thread
// pub fn insert_thread(data: ThreadCreationNotice) -> SendResult {
// 	send_request(Request::InsertThread(data))
// }

// /// Deallocate thread resources and redirect all of its clients
// #[allow(unused)] // TODO: remove this
// pub fn remove_thread(id: u64) -> SendResult {
// 	send_request(Request::RemoveThread(id))
// }

// /// Set the body of an open post
// pub fn set_open_body(post: u64, thread: u64, body: String) -> SendResult {
// 	send_request(Request::SetOpenBody { post, thread, body })
// }

// /// Insert an image into an allocated post
// pub fn insert_image(thread: u64, post: u64, img: Image) -> SendResult {
// 	send_request(Request::InsertImage(ImageInsertionReq {
// 		thread: thread,
// 		post: post,
// 		img: img,
// 	}))
// }

// /// Initialize a freshly-created post
// pub fn insert_post(data: PostCreationNotification) -> SendResult {
// 	send_request(Request::InsertPost(data))
// }
