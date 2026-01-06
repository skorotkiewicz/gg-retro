use tokio_util::sync::CancellationToken;
use tokio::net::TcpListener;
use crate::core::SharedAppState;

mod session;
mod presence;
mod messages;
mod contact_book;

pub use messages::{MessageDispatcher, MessageDispatcherError, MessagesStream, SessionMessage};
pub use presence::{PresenceHub, UserPresence, PresenceChangeStream};

pub async fn gg_server(
  listener: TcpListener,
  shutdown: CancellationToken,
  app_state: SharedAppState
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
  tracing::info!(bind = ?listener.local_addr()?, "Listening GG");

  loop {
    tokio::select! {
      // Check for a shutdown signal
      _ = shutdown.cancelled() => {
        tracing::info!("GG server shutting down...");
        break;
      }

      // Accept new connections
      result = listener.accept() => {
        match result {
          Ok((socket, addr)) => {
            tracing::info!("Accepted connection from {}", addr);
            let conn_shutdown = shutdown.clone();
            let conn_app_state = app_state.clone();
            tokio::spawn(async move {
              let mut session = session::UserSessionController::new(
                socket,
                conn_shutdown,
                conn_app_state
              );

              let result = async {
                session.establish_session().await?;
                session.sync().await?;
                session.run().await
              }.await;

              if let Err(e) = result {
                tracing::error!("Session failed for {}: {}", addr, e);
              }

              session.cleanup().await;
            });
          },
          Err(e) => {
            tracing::error!("Error accepting connection: {}", e);
            return Err(e.into());
          }
        }
      }
    }
  }

  Ok(())
}