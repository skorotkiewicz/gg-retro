use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use gg_protocol::{GGCodec, GGPacket, GGLogin60};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let subscriber = tracing_subscriber::FmtSubscriber::new();
  tracing::subscriber::set_global_default(subscriber)?;

  tracing::info!("Connecting to server...");
  let stream = TcpStream::connect("127.0.0.1:8074").await?;
  tracing::info!("Connected to server");
  let mut protocol = Framed::new(stream, GGCodec::default());

  while let Some(packet) = protocol.next().await {
    tracing::info!("Received packet: {:?}", packet);

    match packet {
      Ok(GGPacket::Welcome { seed }) => {
        tracing::info!("Received welcome message with seed: {}", seed);

        let login_packet = GGLogin60::login(12345, seed, "password");
        protocol.send(GGPacket::Login60(login_packet)).await?;
      },
      Ok(GGPacket::LoginOk) => {
        tracing::info!("Login successful!");
      },
      Ok(GGPacket::LoginFailed) => {
        tracing::error!("Login failed!");
        break;
      },
      _ => {}
    }
  }
  Ok(())
}