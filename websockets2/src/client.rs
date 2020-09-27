use crate::{
	db,
	registry::Registry,
	str_err,
	util::{self, DynResult},
};
use actix::prelude::*;
use actix_web_actors::ws;
use protocol::{
	debug_log,
	payloads::{
		post_body::TextPatch, Authorization, HandshakeReq, PostCreationReq,
		Signature, ThreadCreationReq,
	},
	Decoder, Encoder, MessageType,
};
use serde::{Deserialize, Serialize};
use std::future::Future;

/// Public key public and private ID set
#[derive(Clone, Default, Debug)]
struct PubKeyDesc {
	/// Public key private ID used to sign messages by the client
	priv_id: u64,

	/// Public key public ID used to sign messages by the client
	pub_id: uuid::Uuid,
}

/// Client connection state
#[derive(Debug)]
enum ConnState {
	/// Freshly established a WS connection
	Connected,

	/// Sent handshake message and it was accepted
	AcceptedHandshake,

	/// Public key already registered. Requested client to send a HandshakeReq
	/// with Authorization::Saved.
	RequestedReshake { pub_key: Vec<u8> },

	/// Client synchronizing to a feed
	Synchronizing,
}

#[derive(Debug)]
struct OpenPost {
	id: u64,
	thread: u64,
	body: Vec<char>,
}

impl OpenPost {
	fn new(id: u64, thread: u64) -> Self {
		Self {
			id,
			thread,
			body: Default::default(),
		}
	}
}

/// Maps to a websocket client on the Go side
#[derive(Debug)]
pub struct Client {
	/// ID of client used in various registries
	id: u64,

	/// Client connection state
	conn_state: ConnState,

	/// Post the client is currently editing
	open_post: Option<OpenPost>,

	/// First synchronization of this client occurred
	synced_once: bool,

	/// Public key public and private ID set
	pub_key: PubKeyDesc,

	/// Address to the global registry
	registry: Addr<Registry>,
}

impl Actor for Client {
	type Context = ws::WebsocketContext<Self>;

	fn started(&mut self, ctx: &mut Self::Context) {
		self.registry
			.send(crate::registry::RegisterClient {
				id: self.id,
				addr: ctx.address(),
			})
			.into_actor(self)
			.then(|res, _, ctx| {
				if matches! {res, Err(_)} {
					ctx.stop();
				}
				fut::ready(())
			})
			.wait(ctx);
	}

	fn stopping(&mut self, _: &mut Self::Context) -> actix::Running {
		self.registry
			.do_send(crate::registry::UnregisterClient(self.id));
		actix::Running::Stop
	}
}

impl actix::StreamHandler<Result<ws::Message, ws::ProtocolError>> for Client {
	fn handle(
		&mut self,
		msg: Result<ws::Message, ws::ProtocolError>,
		ctx: &mut Self::Context,
	) {
		todo!("handle messages")
	}
}

/// Return with invalid length error
macro_rules! err_invalid_length {
	($val:expr, $len:expr) => {
		str_err!("invalid {} length: {}", stringify!($val), $len);
	};
}

/// Assert collection length
///
/// $val: expression to check length of
/// $min: minimum length; defaults to 1
/// $max: maximum length
#[rustfmt::skip]
macro_rules! check_len {
	($val:expr, $max:expr) => {
		check_len!($val, 1, $max)
	};
	($val:expr, $min:expr, $max:expr) => {{
		let l = $val.len();
		if l < $min || l > $max {
			err_invalid_length!($val, l)
		}
	}};
}

/// Assert unicode string character length. Returns the length.
///
/// $val: expression to check length of
/// $min: minimum length; defaults to 1
/// $max: maximum length
#[rustfmt::skip]
macro_rules! check_unicode_len {
	($val:expr, $max:expr) => {
		check_unicode_len!($val, 1, $max)
	};
	($val:expr, $min:expr, $max:expr) => {{
		let l = $val.chars().count();
		if l < $min || l > $max {
			err_invalid_length!($val, l)
		}
		l
	}};
}

macro_rules! log_msg_in {
	($type:expr, $msg:expr) => {
		debug_log!(format!(">>> {:?}: {:?}", $type, $msg))
	};
}

impl Client {
	/// Create fresh unconnected client
	pub fn new(registry: Addr<Registry>) -> Self {
		lazy_static::lazy_static! {
			static ref ID_GEN: util::IDGenerator = Default::default();
		}

		Self {
			id: ID_GEN.next(),
			conn_state: ConnState::Connected,
			open_post: Default::default(),
			synced_once: Default::default(),
			pub_key: Default::default(),
			registry,
		}
	}

	/// Decode a message and return the decoded type
	fn decode<T>(t: MessageType, dec: &mut Decoder) -> DynResult<T>
	where
		T: for<'de> serde::Deserialize<'de> + std::fmt::Debug,
	{
		let payload: T = dec.read_next()?;
		log_msg_in!(t, payload);
		Ok(payload)
	}

	/// Handle received message
	async fn receive_message(
		&mut self,
		ctx: &mut <Self as Actor>::Context,
		buf: &[u8],
	) -> DynResult {
		let mut dec = Decoder::new(buf)?;

		let mut first = true;
		loop {
			match dec.peek_type() {
				None => {
					if first {
						str_err!("empty message received");
					}
					return Ok(());
				}
				Some(t) => {
					use ConnState::*;
					use MessageType::*;

					#[rustfmt::skip]
						macro_rules! expect {
							($type:tt) => {
								if t != $type {
									str_err!(concat!(
										"expected ",
										stringify!($type)
									));
								}
							};
						}

					first = false;
					match &self.conn_state {
						Connected => {
							expect!(Handshake);
							self.handle_handshake(&mut dec)?;
						}
						RequestedReshake { pub_key } => {
							expect!(Handshake);
							let pk = pub_key.clone();
							self.handle_reshake(&mut dec, &pk)?;
						}
						AcceptedHandshake => {
							expect!(Synchronize);
							let feed = Self::decode(t, &mut dec)?;
							self.conn_state = Synchronizing;
							self.synchronize(ctx, feed).await?;
						}
						Synchronizing => {
							self.handle_after_synchronizing(t, &mut dec).await?
						}
					}
				}
			}
		}
	}

	/// Handle a received message after reaching ConnState::Synchronizing
	async fn handle_after_synchronizing(
		&mut self,
		t: MessageType,
		dec: &mut Decoder,
	) -> DynResult {
		match t {
			_ => str_err!("unhandled message type: {:?}", t),
		}

		// route! { t,
		// 	InsertThread => |req: ThreadCreationReq| {
		// 		self.insert_thread(req)
		// 	}
		// 	Synchronize => |feed: u64| {
		// 		self.synchronize(feed)
		// 	}
		// 	InsertPost => |req: PostCreationReq| {
		// 		self.insert_post(req)
		// 	}
		// 	Append => |ch: char| {
		// 		self.update_body(1, |b| {
		// 			b.push(ch);
		// 			Ok(())
		// 		})
		// 	}
		// 	Backspace => |_: ()| {
		// 		self.update_body(1, |b| {
		// 			b.pop();
		// 			Ok(())
		// 		})
		// 	}
		// 	PatchPostBody => |req: TextPatch| {
		// 		self.patch_body(req)
		// 	}
		// }
	}

	/// Send a private message to only this client
	fn send<T>(
		&self,
		ctx: &mut <Self as Actor>::Context,
		t: MessageType,
		payload: &T,
	) -> std::io::Result<()>
	where
		T: Serialize + std::fmt::Debug,
	{
		debug_log!(format!("<<< {:?}: {:?}", t, payload));

		let mut enc = Encoder::new(Vec::new());
		enc.write_message(t, payload)?;
		ctx.binary(enc.finish()?);
		Ok(())
	}

	/// Synchronize to a specific thread or board index
	async fn synchronize(
		&mut self,
		ctx: &mut <Self as Actor>::Context,
		feed: u64,
	) -> DynResult {
		if feed != 0 && !db::thread_exists(feed).await? {
			str_err!("invalid thread: {}", feed);
		}

		// Thread init data will be sent on the next Pulsar pulse
		self.registry
			.send(crate::registry::SetFeed {
				client: self.id,
				feed,
			})
			.await?;

		if !self.synced_once {
			self.synced_once = true;
			self.send(ctx, MessageType::CurrentTime, &Self::now())?;
		}
		self.send(ctx, MessageType::Synchronize, &feed)?;

		Ok(())
	}

	/// Validates a solved captcha
	pub fn check_captcha(&mut self, solution: &[u8]) -> DynResult {
		if crate::config::read(|c| c.captcha) {
			check_len!(solution, 4);

			// TODO: Use pub key for spam detection bans
			// TODO: validate solution
		}
		Ok(())
	}

	/// Trim and replace String
	fn trim(src: &mut String) {
		let t = src.trim();
		// Don't always reallocate
		if src.len() != t.len() {
			*src = t.into();
		}
	}

	/// Assert client does not already have an open post
	fn assert_no_open_post(&self) -> Result<(), String> {
		if self.open_post.is_some() {
			str_err!("already have open post")
		}
		Ok(())
	}

	/// Create a new thread and pass its ID to client
	async fn insert_thread(&mut self, mut req: ThreadCreationReq) -> DynResult {
		// TODO: Lock new thread form, if postform is open
		self.assert_no_open_post()?;

		Self::trim(&mut req.subject);
		check_unicode_len!(req.subject, 100);

		check_len!(req.tags, 3);
		for tag in req.tags.iter_mut() {
			Self::trim(tag);
			*tag = tag.to_lowercase();
			check_unicode_len!(tag, 20);
		}
		if req
			.tags
			.iter()
			.collect::<std::collections::BTreeSet<_>>()
			.len() != req.tags.len()
		{
			str_err!("tag set contains duplicates")
		}
		req.tags.sort();

		let [name, trip] = Self::parse_name(req.opts.name)?;
		self.check_captcha(&req.captcha_solution)?;
		let id = db::insert_thread(&mut db::ThreadInsertParams {
			subject: &req.subject,
			tags: &mut req.tags,
			op: db::PostInsertParams {
				public_key: self.pub_key.priv_id.into(),
				name: &name,
				trip: &trip,
				flag: &None, // TODO
				body: &protocol::payloads::post_body::Node::Empty,
			},
		})
		.await?;

		// Ensures old post non-existence records do not persist indefinitely.
		crate::body::cache_location(id, id, 0);

		pulsar::insert_thread(protocol::payloads::ThreadCreationNotice {
			id: id,
			subject: req.subject,
			tags: req.tags,
			time: Self::now(),
		})?;

		self.send(MessageType::InsertThreadAck, &id)?;
		self.open_post = OpenPost::new(id, id).into();
		Ok(())
	}

	/// Return current Unix timestamp
	fn now() -> u32 {
		std::time::SystemTime::now()
			.duration_since(std::time::UNIX_EPOCH)
			.unwrap()
			.as_secs() as u32
	}

	/// Create a new post in a thread and pass its ID to client
	fn insert_post(&mut self, req: PostCreationReq) -> DynResult {
		self.assert_no_open_post()?;

		if bindings::need_captcha(self.pub_key.priv_id)? {
			self.send(MessageType::NeedCaptcha, &())?;
			return Ok(());
		}

		let [name, trip] = Self::parse_name(req.opts.name)?;
		let (id, page) = bindings::insert_post(
			req.thread,
			req.thread,
			&name,
			&trip,
			Self::empty_body_json(),
		)?;

		// Ensures old post non-existence records do not persist indefinitely.
		crate::body::cache_location(id, req.thread, page);

		pulsar::insert_post(protocol::payloads::PostCreationNotification {
			id,
			thread: req.thread,
			page,
			time: Self::now(),
		})?;

		self.send(MessageType::InsertPostAck, &id)?;
		self.open_post = OpenPost::new(id, req.thread).into();
		Ok(())
	}

	/// Apply diff to text body
	fn patch_body(&mut self, req: TextPatch) -> DynResult {
		if req.insert.len() > 2000 {
			str_err!("patch too long")
		}
		if req.insert.len() == 0 && req.remove == 0 {
			str_err!("patch is a NOP")
		}
		self.update_body(req.insert.len() + req.remove as usize, |b| {
			if req.position as usize > b.len() {
				return Err(format!(
					"splice position {} exceeds body length {}",
					req.position,
					b.len()
				));
			}
			let end = b.split_off(req.position as usize);
			b.extend(req.insert.iter());
			b.extend(end);
			Ok(())
		})
	}

	/// Update post body, sync to various services and DB and performs error
	/// handling
	//
	/// affected: number of Unicode characters affected by the mutation
	/// modify: modifies text body
	fn update_body(
		&mut self,
		affected: usize,
		modify: impl Fn(&mut Vec<char>) -> Result<(), String>,
	) -> DynResult {
		match &mut self.open_post {
			Some(p) => {
				modify(&mut p.body)?;
				if p.body.len() > 2000 {
					str_err!("body length exceeds bounds")
				}

				pulsar::set_open_body(p.id, p.thread, p.body.iter().collect())?;
				bindings::increment_spam_score(
					self.pub_key.priv_id,
					affected * crate::config::read(|c| c.spam_scores.character),
				);

				Ok(())
			}
			None => Err("no post open".into()),
		}
	}

	fn decode_handshake(dec: &mut Decoder) -> DynResult<HandshakeReq> {
		let req: HandshakeReq = dec.read_next()?;
		log_msg_in!(MessageType::Handshake, req);
		if req.protocol_version != protocol::VERSION {
			str_err!("protocol version mismatch: {}", req.protocol_version);
		}
		Ok(req)
	}

	fn handle_handshake(&mut self, dec: &mut Decoder) -> DynResult {
		match Self::decode_handshake(dec)?.auth {
			Authorization::NewPubKey(pub_key) => {
				check_len!(pub_key, 1 << 10);
				let (priv_id, pub_id, fresh) =
					bindings::register_public_key(&pub_key)?;

				self.pub_key = PubKeyDesc {
					priv_id,
					pub_id: pub_id.clone(),
				};
				if fresh {
					registry::set_client_key(self.id, priv_id);
					self.conn_state = ConnState::AcceptedHandshake;
				} else {
					self.conn_state = ConnState::RequestedReshake { pub_key };
				}

				self.send(
					MessageType::Handshake,
					&protocol::payloads::HandshakeRes {
						need_resend: !fresh,
						id: pub_id,
					},
				)?;
			}
			Authorization::Saved {
				id: pub_id,
				nonce,
				signature,
			} => {
				let (priv_id, pub_key) = bindings::get_public_key(pub_id)?;
				self.pub_key = PubKeyDesc { priv_id, pub_id };
				self.handle_auth_saved(nonce, signature, pub_key.as_ref())?;
			}
		}
		Ok(())
	}

	/// Handle Authorization::Saved in handshake request
	fn handle_auth_saved(
		&mut self,
		nonce: [u8; 32],
		signature: Signature,
		pub_key: &[u8],
	) -> DynResult {
		let pk = openssl::pkey::PKey::from_rsa(
			openssl::rsa::Rsa::public_key_from_der(pub_key)?,
		)?;
		let mut v = openssl::sign::Verifier::new(
			openssl::hash::MessageDigest::sha256(),
			&pk,
		)?;
		v.update(self.pub_key.pub_id.as_bytes())?;
		v.update(&nonce)?;
		if !v.verify(&signature.0)? {
			str_err!("invalid signature");
		}

		self.send(
			MessageType::Handshake,
			&protocol::payloads::HandshakeRes {
				need_resend: false,
				id: self.pub_key.pub_id,
			},
		)?;
		self.conn_state = ConnState::AcceptedHandshake;
		Ok(())
	}

	/// Handle repeated handshake after request by server
	fn handle_reshake(
		&mut self,
		mut dec: &mut Decoder,
		pub_key: &[u8],
	) -> DynResult {
		match Self::decode_handshake(&mut dec)?.auth {
			Authorization::Saved {
				id: pub_id,
				nonce,
				signature,
			} => {
				if pub_id != self.pub_key.pub_id {
					str_err!("different public key public id in reshake");
				}
				self.handle_auth_saved(nonce, signature, pub_key)?;
			}
			_ => str_err!("invalid authorization variant"),
		}
		Ok(())
	}

	/// Parse post name field in to name and tripcode
	fn parse_name(
		mut src: String,
	) -> Result<[Option<String>; 2], &'static str> {
		use tripcode::{FourchanNonescaping, TripcodeGenerator};

		Ok(match src.len() {
			0 => Default::default(),
			l if l > 50 => Err("name too long")?,
			_ => {
				Self::trim(&mut src);
				match src.as_bytes().iter().position(|b| b == &b'#') {
					Some(i) if i != src.len() - 1 => {
						let trip = FourchanNonescaping::generate(&src[i + 1..]);
						src.truncate(i);
						[Some(src), Some(trip)]
					}
					_ => [Some(src), None],
				}
			}
		})
	}
}
