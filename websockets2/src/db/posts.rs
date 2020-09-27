use crate::util::DynResult;
use protocol::payloads::post_body::Node;
use std::{collections::HashMap, sync::Arc};

// Common params for both post and thread insertion
pub struct PostInsertParams<'a> {
	pub public_key: Option<u64>,
	pub name: &'a Option<String>,
	pub trip: &'a Option<String>,
	pub flag: &'a Option<String>,
	pub body: &'a Node,
}

pub async fn write_open_post_bodies(
	bodies: HashMap<u64, Arc<Node>>,
) -> DynResult {
	let mut bodies: Vec<(u64, Arc<Node>)> = bodies.into_iter().collect();

	// Sort by ID for more sequential DB access
	bodies.sort_unstable_by(|(a, _), (b, _)| a.cmp(b));

	let mut cl = super::get_client().await?;
	let tx = cl.transaction().await?;
	let q = tx
		.prepare(
			r"update posts
			set body = $2
			where id = $1 and open = true
			",
		)
		.await?;

	for (id, body) in bodies {
		tx.execute(&q, &[&(id as i64), &tokio_postgres::types::Json(body)])
			.await?;
	}

	tx.commit().await?;
	Ok(())
}
