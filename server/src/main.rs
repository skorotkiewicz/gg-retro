mod api;
mod banner;
mod messenger;
mod core;
mod models;

use tokio::net::TcpListener;
use tokio::signal;
use tokio_util::sync::CancellationToken;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  banner::print_banner();

  let subscriber = tracing_subscriber::FmtSubscriber::builder()
    .finish();
  tracing::subscriber::set_global_default(subscriber)?;
  let app_state = core::create_app_state().await?;

  // Create a shutdown token for graceful shutdown coordination
  let shutdown = CancellationToken::new();

  let gg_listener = TcpListener::bind(app_state.config().gg_bind()).await?;
  let web_listener = TcpListener::bind(app_state.config().api_bind()).await?;

  tracing::info!("GG messenger listening on {}", app_state.config().gg_bind());
  tracing::info!("Web API listening on http://{}", app_state.config().api_bind());

  // Clone tokens for each task
  let messenger_shutdown = shutdown.clone();
  let api_shutdown = shutdown.clone();

  let messenger_task_app_state = app_state.clone();
  let mut messenger_task = tokio::spawn(async move {
    messenger::gg_server(gg_listener, messenger_shutdown, messenger_task_app_state).await
  });

  let api_task_app_state = app_state.clone();
  let mut api_task = tokio::spawn(async move {
    api::http_server(web_listener, api_shutdown, api_task_app_state).await
  });

  // Wait for shutdown signal or task failure
  tokio::select! {
    _ = signal::ctrl_c() => {
      tracing::info!("Received Ctrl+C, initiating graceful shutdown...");
    }
    result = &mut messenger_task => {
      tracing::error!("GG server exited unexpectedly: {:?}", result);
    }
    result = &mut api_task => {
      tracing::error!("HTTP server exited unexpectedly: {:?}", result);
    }
  }

  // Signal all tasks to shut down
  shutdown.cancel();
  tracing::info!("Shutdown complete");
  let _ = messenger_task.await;
  let _ = api_task.await;
  Ok(())
}
