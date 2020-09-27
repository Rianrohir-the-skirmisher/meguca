mod posts;
mod threads;

pub use posts::*;
pub use threads::*;

use crate::util::DynResult;

/// Get a client corresponding to a connection from the connection pool
async fn get_client() -> DynResult<deadpool_postgres::Client> {
	lazy_static::lazy_static! {
		static ref POOL: Result<deadpool_postgres::Pool, String> =
			|| -> DynResult<deadpool_postgres::Pool> {
				use deadpool_postgres::config as cfg;

				let u: tokio_postgres::Config = crate::config::SERVER
												.database.parse()?;
				let conf = cfg::Config {
					user: u.get_user().map(|s| s.into()),
					password: u
						.get_password()
						.map(|b| {
							String::from_utf8(b.iter().copied().collect()).ok()
						})
						.flatten(),
					dbname: u.get_dbname().map(|s| s.into()),
					application_name: u
						.get_application_name()
						.map(|s| s.into()),
					ssl_mode: Some(cfg::SslMode::Disable),
					hosts: u
						.get_hosts()
						.iter()
						.map(|h| {
							use tokio_postgres::config::Host::*;

							match h {
								Tcp(s) => Ok(s.clone()),
								Unix(p) => match p.as_path().to_str() {
									Some(s) => Ok(s.into()),
									None => Err(format!(
										concat!(
											"could not parse Unix ",
											"socket host: {:?}"
										),
										h
									)
									.into()),
								},
							}
						})
						.collect::<Result<Vec<String>, String>>()
						.map(|v| if v.is_empty() { None } else { Some(v) })?,
					ports: {
						let p = u.get_ports();
						if p.is_empty() {
							None
						} else {
							Some(p.iter().copied().collect())
						}
					},
					connect_timeout: u.get_connect_timeout().cloned(),
					keepalives: u.get_keepalives().into(),
					keepalives_idle: u.get_keepalives_idle().into(),
					target_session_attrs: Some(
						cfg::TargetSessionAttrs::ReadWrite,
					),
					..Default::default()
				};

				Ok(conf.create_pool(tokio_postgres::tls::NoTls)?)
			}()
			.map_err(|e| e.to_string())
			.into();
	}

	Ok(POOL.as_ref().map_err(|e| e.clone())?.get().await?)
}
