mod body;
mod client;
mod config;
mod db;
mod feeds;
mod message;
mod registry;
mod util;

use actix::prelude::*;
use actix_web::{get, web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use registry::Registry;

#[get("/api/socket")]
async fn connect(
    req: HttpRequest,
    stream: web::Payload,
    registry: web::Data<Addr<Registry>>,
) -> Result<HttpResponse, Error> {
    ws::start(
        client::Client::new(registry.get_ref().clone()),
        &req,
        stream,
    )
}

// TODO: asset and image routes

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    stderrlog::new().init().unwrap();

    // TODO: run migrations

    // TODO: read a snapshot of the thread index and pass that to the feeds.
    // You can read all of it in one call and there no longer be a need to read
    // the thread IDs.

    fn conv_error(err: util::Err) -> std::io::Error {
        std::io::Error::new(std::io::ErrorKind::Other, err)
    }

    let (feed_data, threads) =
        futures::future::join(db::get_feed_data(), db::thread_ids()).await;
    let feed_data = feed_data.map_err(conv_error)?;
    let threads = threads.map_err(conv_error)?;

    // TODO: spawn global pulsar, registry and body flusher instances
    let registry =
        Registry::create(|ctx| Registry::new(ctx, threads, feed_data));

    HttpServer::new(move || {
        App::new().app_data(registry.clone()).service(connect)
    })
    .bind(&config::SERVER.address)?
    .run()
    .await
}
