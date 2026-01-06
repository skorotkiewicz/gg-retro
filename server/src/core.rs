use std::net::IpAddr;
use std::sync::Arc;
use axum::http;
use axum::http::Uri;
use figment::Figment;
use figment::providers::{Env, Format, Serialized, Toml};
use local_ip_address::local_ip;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};
use crate::messenger::{MessageDispatcher, PresenceHub};
use crate::models::DatabasePool;

#[derive(Deserialize, Serialize, Debug)]
pub struct ServerConfig {
  bind: String,
  http_port: u16,
  gg_port: u16,
  db: String,
  hostname: String,
}

impl Default for ServerConfig {
  fn default() -> Self {
    Self {
      bind: "0.0.0.0".to_string(),
      http_port: 80,
      gg_port: 8074,
      db: "./gg.db".to_string(),
      hostname: "gg-retro.local".to_string(),
    }
  }
}

impl ServerConfig {
  pub fn api_bind(&self) -> String {
    format!("{}:{}", self.bind, self.http_port)
  }

  pub fn gg_bind(&self) -> String {
    format!("{}:{}", self.bind, self.gg_port)
  }

  pub fn hostname(&self) -> &str {
    &self.hostname
  }
}

/// Shared application state containing resources needed across the server.
#[derive(Debug)]
pub struct AppState {
  db_pool: DatabasePool,
  pub host_ip: IpAddr,
  presence_hub: PresenceHub,
  message_dispatcher: MessageDispatcher,
  config: ServerConfig
}

impl AppState {
  /// Get a reference to the database pool.
  pub fn db_pool(&self) -> &DatabasePool {
    &self.db_pool
  }

  pub fn config(&self) -> &ServerConfig {
    &self.config
  }

  pub fn presence_hub(&self) -> &PresenceHub {
    &self.presence_hub
  }

  pub fn message_dispatcher(&self) -> &MessageDispatcher {
    &self.message_dispatcher
  }

  pub fn host_uri(&self, path: &str) -> Result<Uri, http::Error> {
    Ok(Uri::builder()
        .scheme("http")
        .authority(self.config.hostname())
        .path_and_query(path)
        .build()?)
  }
}

pub type SharedAppState = Arc<AppState>;

pub async fn create_app_state() -> Result<SharedAppState, Box<dyn std::error::Error>> {
  let config : ServerConfig = Figment::new()
    .merge(Toml::file("/etc/gg-retro/config.toml"))
    .merge(Toml::file("config.toml"))
    .merge(Env::prefixed("GG_"))
    .join(Serialized::defaults(ServerConfig::default()))
    .extract()?;

  tracing::info!(db = config.db, "Starting GG server, preparing database...");
  let db_options = SqliteConnectOptions::new()
    .filename(config.db.clone())
    .create_if_missing(true)
    .journal_mode(SqliteJournalMode::Wal);

  let db_pool = SqlitePoolOptions::new()
    .max_connections(32)
    .connect_with(db_options)
    .await?;

  sqlx::migrate!("./migrations").run(&db_pool).await?;
  tracing::info!("Database ready");

  let host_ip = local_ip()?;
  let presence_hub = PresenceHub::new();
  let message_dispatcher = MessageDispatcher::new(&db_pool);

  Ok(
    Arc::new(
      AppState {
        db_pool,
        host_ip,
        presence_hub,
        message_dispatcher,
        config
      }
    )
  )
}