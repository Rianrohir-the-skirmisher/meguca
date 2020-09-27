use super::{get_client, PostInsertParams};
use crate::util::DynResult;
use protocol::payloads::FeedData;
use tokio_postgres::types::Json;

/// Parameters for inserting a thread and its OP
pub struct ThreadInsertParams<'a> {
	subject: &'a str,
	tags: &'a mut [String],
	op: PostInsertParams<'a>,
}

pub async fn thread_exists(id: u64) -> DynResult<bool> {
	Ok(get_client()
		.await?
		.query_one(
			r"select exists (
				select
				from threads
				where id = $1
			)",
			&[&(id as i64)],
		)
		.await?
		.get(0))
}

/// Insert thread and empty post into DB and return the thread ID
pub async fn insert_thread<'a>(
	p: &mut ThreadInsertParams<'a>,
) -> DynResult<u64> {
	let mut c = super::get_client().await?;
	let tx = c.transaction().await?;

	p.tags.sort();
	let id: i64 = tx
		.query_one(
			r"insert into threads (subject, tags)
			values ($1, $2)
			returning id",
			&[&p.subject, &(p.tags as &[String])],
		)
		.await?
		.get(0);
	tx.execute(
		r"insert into posts (
			id,
			public_key,
			name,
			trip,
			flag,
			body
		)
		values (
			$1,
			$2,
			$3,
			$4,
			$5,
			$6
		)",
		&[
			&id,
			&(p.op.public_key.map(|i| i as i64)),
			&p.op.name,
			&p.op.trip,
			&p.op.flag,
			&Json(p.op.body),
		],
	)
	.await?;

	Ok(id as u64)
}

/// Read data used to initialize Pulsar on server start
pub async fn get_feed_data() -> DynResult<Vec<FeedData>> {
	Ok(get_client()
		.await?
		.query_one(
			r"select coalesce(
				jsonb_agg(
					jsonb_build_object(
						'thread', t.id,
						'recent_posts', coalesce(r.val, '{}'::jsonb),
						'open_posts', coalesce(o.val, '{}'::jsonb)
					)
				),
				'[]'::jsonb
			)
			from threads t
			left join (
				select
					r.thread,
					jsonb_object_agg(
						r.id,
						to_unix(r.created_on)
					) val
				from posts r
				where r.created_on > now() - interval '16 minutes'
				group by r.thread
			) r on r.thread = t.id
			left join (
				select
					o.thread,
					jsonb_object_agg(
						o.id,
						jsonb_build_object(
							'has_image', o.image is not null,
							'image_spoilered', o.image_spoilered,
							'created_on', to_unix(o.created_on),
							'thread', o.thread,
							'body', o.body
						)
					) val
				from posts o
				where o.open
				group by o.thread
			) o on o.thread = t.id",
			&[],
		)
		.await?
		.get::<'_, _, Json<Vec<FeedData>>>(0)
		.0)
}

/// Return all existing thread IDs
pub async fn thread_ids() -> DynResult<Vec<u64>> {
	Ok(get_client()
		.await?
		.query("select id from threads", &[])
		.await?
		.into_iter()
		.map(|r| r.get::<'_, _, i64>(0) as u64)
		.collect())
}
