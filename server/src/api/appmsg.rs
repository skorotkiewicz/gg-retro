//! Server discovery endpoints.
//!
//! These endpoints return the GG server address for clients to connect to.
//! - `/appsvc/appmsg4.asp` - Plain TCP connection (port 8074)
//! - `/appsvc/appmsg3.asp` - TLS connection (port 443)

use axum::{Router, routing::get, extract::Query};
use axum::extract::State;
use axum::http::StatusCode;
use serde::Deserialize;
use crate::core::SharedAppState;

/// Query parameters for server discovery.
#[derive(Debug, Deserialize)]
pub struct AppMsgParams {
  /// User's GG number (UIN).
  pub fmnumber: Option<u32>,
  /// Client version in format "A, B, C, D".
  pub version: Option<String>,
  /// Response format: None for plain text, "2" for HTML.
  pub fmt: Option<String>,
  /// Last received system message number.
  pub lastmsg: Option<u32>,
}

/// Server discovery for plain TCP connections.
///
/// Response format: `MSG_NUM 0 IP:PORT IP`
/// Example: `0 0 217.17.41.84:8074 217.17.41.84`
#[tracing::instrument]
pub async fn appmsg4(Query(params): Query<AppMsgParams>, State(app_state) : State<SharedAppState>) -> Result<String, StatusCode> {
  // request{method=GET uri=/appsvc/appmsg4.asp?fmnumber=5000&version=6%2C+1%2C+0%2C+158&fmt=2&lastmsg=26679 version=HTTP/1.0}: tower_http::trace::on_response: finished processing request latency=0 ms status=200

  Ok(format!("0 0 {}:8074 {}", app_state.host_ip, app_state.host_ip).into())
}

/// Server discovery for TLS connections.
///
/// Returns empty 200 response (TLS not implemented).
#[tracing::instrument]
pub async fn appmsg3(Query(_params): Query<AppMsgParams>) -> StatusCode {
  StatusCode::OK
}

pub fn router() -> Router<SharedAppState> {
  Router::new()
    .route("/appsvc/appmsg4.asp", get(appmsg4))
    .route("/appsvc/appmsg3.asp", get(appmsg3))
}
