//! Password recovery endpoint.
//!
//! `/appsvc/fmsendpwd3.asp` - Sends password reminder to registered email.

use axum::{Router, routing::post, Form};
use serde::Deserialize;
use crate::core::SharedAppState;

/// Form fields for password recovery.
#[derive(Debug, Deserialize)]
pub struct SendPwdForm {
  /// GG number (UIN).
  pub userid: u32,
  /// Token ID from regtoken endpoint.
  pub tokenid: String,
  /// User-entered token value (captcha answer).
  pub tokenval: String,
  /// Hash code for verification.
  pub code: String,
}

/// Send password reminder to registered email.
///
/// Success response: `pwdsend_success`
#[tracing::instrument]
pub async fn sendpwd(Form(form): Form<SendPwdForm>) -> String {
  todo!("Implement password recovery")
}

pub fn router() -> Router<SharedAppState> {
  Router::new()
    .route("/appsvc/fmsendpwd3.asp", post(sendpwd))
}
