//! Account management endpoint.
//!
//! `/appsvc/fmregister3.asp` - Handles account registration, deletion, and password changes.

use axum::{Router, routing::post, Form, extract::State};
use serde::Deserialize;
use crate::core::SharedAppState;
use crate::models::{RepositoryError, TokenRepository, UserRepository};

/// Form fields for account operations.
#[derive(Debug, Deserialize)]
pub struct RegisterForm {
  /// New password (for registration/change).
  pub pwd: Option<String>,
  /// Email address.
  pub email: Option<String>,
  /// Token ID from regtoken endpoint.
  pub tokenid: Option<String>,
  /// User-entered token value (captcha answer).
  pub tokenval: Option<String>,
  /// Hash code for verification.
  pub code: Option<String>,
  /// GG number (for existing account operations).
  pub fmnumber: Option<u32>,
  /// Current password (for account changes).
  pub fmpwd: Option<String>,
  /// Set to "1" to delete account.
  pub delete: Option<String>,
}

/// Handle account registration, deletion, or password change.
///
/// **Registration:**
/// - Required: `pwd`, `email`, `tokenid`, `tokenval`, `code`
/// - Success: `reg_success:UIN`
///
/// **Deletion:**
/// - Required: `fmnumber`, `fmpwd`, `delete=1`, `tokenid`, `tokenval`, `code`
///
/// **Password Change:**
/// - Required: `fmnumber`, `fmpwd`, `pwd`, `email`, `tokenid`, `tokenval`, `code`
#[tracing::instrument(skip(app_state))]
pub async fn register(
  State(app_state): State<SharedAppState>,
  Form(form): Form<RegisterForm>
) -> Result<String, RepositoryError> {
  let users = UserRepository::new(app_state.db_pool());
  let tokens = TokenRepository::new(app_state.db_pool());

  // Registration: no fmnumber means new account
  if form.fmnumber.is_none() {
    if let (Some(pwd), Some(email), Some(tokenid), Some(captcha_code)) = (form.pwd, form.email, form.tokenid, form.tokenval) {
      if !tokens.validate(&tokenid, &captcha_code).await? {
        tracing::error!("invalid captcha");
        return Ok("reg_failed".to_string())
      }

      let name = email.split('@').next().unwrap_or("user");
      let user = users.create(name, &email, &pwd).await?;

      tracing::info!(uin = user.uin, email = %email, "User registered");
      return Ok(format!("reg_success:{}", user.uin))
    }
  }

  // TODO: Implement deletion (fmnumber + fmpwd + delete="1")
  // TODO: Implement password change (fmnumber + fmpwd + pwd + email)

  Ok("reg_failed".to_string())
}

pub fn router() -> Router<SharedAppState> {
  Router::new()
    .route("/appsvc/fmregister3.asp", post(register))
}
