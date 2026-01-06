use std::collections::HashMap;
use std::time::Duration;
use thiserror::Error;
use tokio::sync::{mpsc, RwLock};
use tokio::sync::mpsc::error::SendError;
use tokio::time::error::Elapsed;
use tokio_stream::wrappers::ReceiverStream;
use tracing::instrument;
use gg_protocol::consts::AckStatus;
use gg_protocol::GGNumber;
use gg_protocol::packets::{GGRecvMessage, GGSendMessage};
use crate::models::{DatabasePool, MessageRepository, RepositoryError, UserRepository, QueuedMessageId};

#[derive(Clone, Debug)]
pub enum SessionMessage {
  Disconnect,
  QueuedMessage(QueuedMessageId)
}

pub type MessagesStream = ReceiverStream<SessionMessage>;

#[derive(Debug)]
pub struct MessageDispatcher {
  sessions: RwLock<HashMap<GGNumber, mpsc::Sender<SessionMessage>>>,
  db_pool: DatabasePool,
}

#[derive(Error, Debug)]
pub enum MessageDispatcherError {
  #[error("Failed to deliver message to recipient session")]
  MessageDeliveryFailed(#[from] SendError<SessionMessage>),
  #[error("Failed to store message in database")]
  StorageError(#[from] RepositoryError),
  #[error("Failed to send message to recipient session")]
  SendTimeout(#[from] Elapsed)
}

impl MessageDispatcher {
  pub fn new(db_pool: &DatabasePool) -> Self {
    Self {
      sessions: RwLock::new(HashMap::new()),
      db_pool: db_pool.clone()
    }
  }

  #[instrument(skip(self))]
  pub async fn register(&self, uin: GGNumber) -> MessagesStream {
    let mut sessions = self.sessions.write().await;
    let (tx, rx) = mpsc::channel::<SessionMessage>(100);
    sessions.insert(uin, tx);
    ReceiverStream::new(rx)
  }

  #[instrument(skip(self))]
  pub async fn unregister(&self, uin: GGNumber) {
    let mut sessions = self.sessions.write().await;
    sessions.remove(&uin);
  }

  #[instrument(skip(self))]
  pub async fn kick(&self, uin: GGNumber) {
    let mut sessions = self.sessions.write().await;
    if let Some(sender) = sessions.get(&uin) {
      tracing::info!(uin = uin, "User already signed in, kicking other session");
      let _ = sender.send(SessionMessage::Disconnect).await;
      sessions.remove(&uin);
      tokio::time::sleep(Duration::from_millis(10)).await;// yield thread
    }
  }

  #[instrument(skip(self))]
  pub async fn dispatch(&self, sender: GGNumber, incoming_msg : GGSendMessage) -> Result<AckStatus, MessageDispatcherError> {
    let messages = MessageRepository::new(&self.db_pool);
    let users = UserRepository::new(&self.db_pool);
    let sessions = self.sessions.read().await;
    let recipient = sessions.get(&incoming_msg.recipient);
    let now = std::time::SystemTime::now()
      .duration_since(std::time::UNIX_EPOCH)
      .unwrap_or_default()
      .as_secs() as u32;
    
    if !users.exists(sender).await? || !users.exists(incoming_msg.recipient).await? {
      tracing::error!(sender = sender, recipient = incoming_msg.recipient, "message dispatch failed: one or both users do not exist");
      return Ok(AckStatus::NotDelivered)
    }

    let recv_msg = GGRecvMessage {
      message: incoming_msg.message,
      class: incoming_msg.class,
      seq: incoming_msg.seq,
      time: now,
      sender,
      formatting: incoming_msg.formatting,
    };

    let msg = messages.store(incoming_msg.recipient, &recv_msg).await?;

    if let Some(tx) = recipient {
      let _ = tokio::time::timeout(
        Duration::from_secs(5),
        tx.send(SessionMessage::QueuedMessage(msg.id))
      ).await?;
      tracing::info!(msg_id = msg.id, "Message send for delivery");
      Ok(AckStatus::Delivered)
    } else {
      tracing::info!(msg_id = msg.id, "Message put to queue, user offline");
      Ok(AckStatus::Queued)
    }
  }
}
