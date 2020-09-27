use crate::registry::Registry;
use actix::prelude::*;

/// Feed for the thread index
#[derive(Debug)]
pub struct IndexFeed {
	registry: Addr<Registry>,
}

impl Actor for IndexFeed {
	type Context = Context<Self>;
}

impl IndexFeed {
	pub fn new(registry: Addr<Registry>) -> Self {
		Self { registry }
	}
}
