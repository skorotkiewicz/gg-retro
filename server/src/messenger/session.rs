use std::fmt::{Debug, Formatter};
use std::time::Duration;
use futures::{SinkExt, StreamExt};
use rand::Rng;
use thiserror::Error;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use tokio_util::sync::CancellationToken;
use tracing::instrument;
use gg_protocol::{GGCodec, GGError, GGPacket, GGNumber};
use gg_protocol::packets::{ContactEntry, ContactStatus, ContactType, GGSendMessageAck};
use crate::core::SharedAppState;
use crate::messenger::{MessageDispatcherError, MessagesStream, PresenceChangeStream, SessionMessage, UserPresence};
use crate::messenger::contact_book::ContactBook;
use crate::models::{MessageRepository, UserRepository};

pub struct UserSessionController {
  seed: u32,
  uin: Option<GGNumber>,
  initial_presence: Option<UserPresence>,
  contacts: ContactBook,
  protocol: Framed<TcpStream, GGCodec>,
  peer_addr: std::net::SocketAddr,
  shutdown: CancellationToken,
  app_state: SharedAppState,
  presence_change_stream: Option<PresenceChangeStream>,
  messages_stream: Option<MessagesStream>
}

impl Debug for UserSessionController {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("UserSessionController")
      .field("uin", &self.uin)
      .field("peer_addr", &self.peer_addr)
      .finish()
  }
}

#[derive(Error, Debug)]
pub enum UserSessionError {
  #[error("Client disconnected because of server shutdown")]
  ServerShutdown,
  #[error("Client disconnected")]
  ClientDisconnected,
  #[error("Authentication failed to finish in time")]
  AuthenticateTimeout,
  #[error("Authentication failed: invalid credentials")]
  InvalidCredentials,
  #[error("GG protocol error: {0}")]
  ProtocolError(#[from] GGError),
  #[error("Session timed out")]
  SessionTimeout,
  #[error("Database error: {0}")]
  DatabaseError(#[from] sqlx::Error),
  #[error("Repository error: {0}")]
  RepositoryError(#[from] crate::models::RepositoryError),
  #[error("Failed to deliver message: {0}")]
  DeliveryError(#[from] MessageDispatcherError)
}

impl UserSessionController {
  #[instrument(skip(stream, app_state))]
  pub fn new(stream: TcpStream, shutdown: CancellationToken, app_state: SharedAppState) -> Self {
    let peer_addr = stream.peer_addr().expect("Well no ip in this stream WTF");
    let protocol = Framed::new(stream, GGCodec::server());
    let seed = rand::rng().random_range(100_000..1_000_000);

    Self {
      peer_addr,
      seed,
      contacts: ContactBook::new(),
      uin: None,
      protocol,
      app_state,
      shutdown,
      presence_change_stream: None,
      messages_stream: None,
      initial_presence: None
    }
  }

  #[instrument]
  pub async fn establish_session(&mut self) -> Result<(), UserSessionError> {
    tracing::info!("Sending welcome packet...");
    self.protocol.send(GGPacket::Welcome {
      seed: self.seed
    }).await?;

    let timeout_task = tokio::time::sleep(Duration::from_secs(60));
    tokio::pin!(timeout_task);

    loop {
      tokio::select! {
        _ = self.shutdown.cancelled() => {
          tracing::debug!("Connection {} closing due to shutdown", self.peer_addr);
          self.protocol.send(GGPacket::Disconnect).await?;
          return Err(UserSessionError::ServerShutdown);
        },

        _ = &mut timeout_task => {
          tracing::error!("Authentication timed out for {}", self.peer_addr);
          self.protocol.send(GGPacket::Disconnect).await?;
          return Err(UserSessionError::AuthenticateTimeout);
        },

        result = self.protocol.next() => {
          match result {
            None => {
              tracing::info!("Connection closed from {}", self.peer_addr);
              return Err(UserSessionError::ClientDisconnected);
            },

            Some(Err(e)) => {
              tracing::error!("Error reading packet: {}", e);
              return Err(UserSessionError::ProtocolError(e));
            },

            Some(Ok(GGPacket::Login60(login_info))) => {
              tracing::info!("Received login from UIN: {}", login_info.uin);
              let users = UserRepository::new(self.app_state.db_pool());

              let user = users.find_by_uin(login_info.uin).await?;

              if let Some(user) = user {
                tracing::info!("User found for UIN: {}", login_info.uin);
                let expected_password = gg_protocol::gg_login_hash(&user.password, self.seed);

                if login_info.hash == expected_password {
                  tracing::info!("Authentication successful for {}", self.peer_addr);
                  let uin = login_info.uin;
                  self.uin = Some(uin);
                  self.initial_presence = Some(login_info.into());
                  tracing::info!(presence = ?self.initial_presence, uin = uin, "Initial user presence");

                  self.protocol.send(GGPacket::LoginOk).await?;
                  return Ok(())
                } else {
                  tracing::error!("Invalid password for {}", login_info.uin);
                  self.protocol.send(GGPacket::LoginFailed).await?;
                  return Err(UserSessionError::InvalidCredentials)
                }
              } else {
                tracing::error!("User not found for UIN: {}", login_info.uin);
                self.protocol.send(GGPacket::LoginFailed).await?;
                return Err(UserSessionError::InvalidCredentials);
              }
            },

            Some(Ok(other)) => tracing::warn!("ignoring packet type: {:?}", other)
          }
        }
      }
    }
  }

  #[instrument]
  pub async fn sync(&mut self) -> Result<(), UserSessionError> {
    let current_uin = self.uin.expect("Missing uin");

    self.app_state.message_dispatcher().kick(current_uin).await;
    self.presence_change_stream = Some(self.app_state.presence_hub().register(current_uin));
    self.messages_stream = Some(self.app_state.message_dispatcher().register(current_uin).await);

    let presence = self.initial_presence.take().unwrap_or_else(|| UserPresence::available(current_uin));
    self.app_state.presence_hub().notify(presence);

    Ok(())
  }

  #[instrument]
  async fn deliver_pending_messages(&mut self) -> Result<(), UserSessionError> {
    let current_uin = self.uin.expect("Missing uin");

    let messages = MessageRepository::new(self.app_state.db_pool());

    tracing::info!(uin = ?current_uin, "Starting pending message synchronization");
    let mut total_delivered = 0u32;

    while let Some(pending_messages) = messages.find_pending(current_uin).await? {
      let batch_size = pending_messages.len();
      tracing::debug!(uin = ?current_uin, batch_size, "Processing pending message batch");

      let mut delivered = Vec::new();
      for msg in pending_messages {
        let msg_id = msg.id;
        if self.contacts.is_blocked(msg.sender_uin) {
          tracing::debug!(msg_id, sender = msg.sender_uin, uin = ?current_uin, "blocking pending message");
        } else {
          tracing::debug!(msg_id, sender = msg.sender_uin, uin = ?current_uin, "Delivering pending message");
          self.protocol.send(GGPacket::RecvMessage(msg.into())).await?;
        }
        delivered.push(msg_id);
      }

      messages.mark_delivered(&delivered).await?;
      total_delivered += batch_size as u32;
      tracing::debug!(uin = ?current_uin, batch_size, total_delivered, "Batch delivered and marked");
    }

    tracing::info!(uin = ?current_uin, total_delivered, "Pending message synchronization complete");
    Ok(())
  }

  #[instrument]
  pub async fn run(&mut self) -> Result<(), UserSessionError> {
    let mut contacts: Vec<ContactEntry> = Vec::new();
    let current_uin = self.uin.expect("Missing uin");
    tracing::info!("Starting user session for {}", current_uin);

    loop {
      let presence_change_stream = self.presence_change_stream.as_mut().expect("Presence subscription not set");
      let messages_stream = self.messages_stream.as_mut().expect("Message subscription not set");

      tokio::select! {
        _ = self.shutdown.cancelled() => {
          tracing::info!("User session {} shutting down...", current_uin);
          self.protocol.send(GGPacket::Disconnect).await?;
          self.protocol.flush().await?;
          break;
        },

        _ = tokio::time::sleep(Duration::from_mins(5)) => {
          tracing::error!("User timeout");
          return Err(UserSessionError::SessionTimeout);
        },

        Some(session_msg) = messages_stream.next() => {
          match session_msg {
            SessionMessage::Disconnect => {
              tracing::info!(uin = ?current_uin, "already signed in, kicking current session");
              self.protocol.send(GGPacket::Disconnect).await?;
              break;
            },

            SessionMessage::QueuedMessage(msg_id) => {
              tracing::info!(msg_id = msg_id, uin = ?current_uin, "delivering message");

              let messages = MessageRepository::new(self.app_state.db_pool());
              if let Some(message) = messages.find_one_pending(msg_id).await? {
                if self.contacts.is_blocked(message.sender_uin) {
                  tracing::error!(sender = message.sender_uin, uin = current_uin, "is blocked, skipping message delivery");
                } else {
                  self.protocol.send(GGPacket::RecvMessage(message.into())).await?;
                }

                messages.mark_single_delivered(msg_id).await?;
              } else {
                tracing::error!(msg_id = msg_id, "message already delivered");
              }
            }
          }
        },

        Some(uin) = presence_change_stream.next() => {
          let presence = self.app_state.presence_hub().find(&uin);
          tracing::info!(presence = ?presence, uin = ?current_uin, "Presence changed, sending new presence to client");
          self.protocol.send(GGPacket::Status60(presence.into())).await?;
        },

        result = self.protocol.next() => {
          match result {
            None => {
              tracing::info!("Connection closed from {}", current_uin);
              return Err(UserSessionError::ClientDisconnected);
            },

            Some(Ok(GGPacket::Disconnect)) => {
              tracing::info!("Connection closed from {}", current_uin);
              return Err(UserSessionError::ClientDisconnected);
            },

            Some(Ok(GGPacket::SendMessage(incoming_message))) => {
              tracing::info!(incoming_message = ?incoming_message, uin = current_uin, "received message, relaying it to recipient");

              let recipient = incoming_message.recipient;
              let seq = incoming_message.seq;
              let status = self.app_state.message_dispatcher().dispatch(current_uin, incoming_message).await?;
              self.protocol.send(GGPacket::SendMessageAck(GGSendMessageAck { seq, status, recipient })).await?;
            },

            Some(Ok(GGPacket::NewStatus(new_status))) => {
              tracing::info!(uin = current_uin, status = ?new_status, "client changed status, sending info to other contacts");
              let presence = UserPresence {
                status: new_status.status,
                uin: current_uin,
                description: new_status.description,
                time: new_status.time
              };
              self.app_state.presence_hub().notify(presence);
            },

            Some(Ok(GGPacket::ListEmpty)) => {
              tracing::info!(uin = current_uin, "client has empty contact list");
              self.deliver_pending_messages().await?;
            },

            Some(Ok(GGPacket::Ping)) => {
              tracing::info!(uin = current_uin, "received ping, sending pong");
              self.protocol.send(GGPacket::Pong).await?;
            },

            Some(Ok(GGPacket::NotifyFirst(new_contacts))) => {
              tracing::info!(uin = current_uin, count = new_contacts.len(), "received first batch of contacts");
              contacts.extend(new_contacts);
            },

            Some(Ok(GGPacket::NotifyLast(last_contacts))) => {
              tracing::info!(uin = current_uin, count = last_contacts.len(), total = contacts.len() + last_contacts.len(), "received last batch of contacts, processing contact list");
              contacts.extend(last_contacts);
              self.handle_contact_list(&contacts).await?;
              contacts.clear();
              self.deliver_pending_messages().await?;
            },

            _ => {
              tracing::info!("Ignoring packet type: {:?}", result);
            }
          }
        }
      }
    }

    return Ok(())
  }

  #[instrument]
  async fn handle_contact_list(&mut self, contacts: &Vec<ContactEntry>) -> Result<(), UserSessionError> {
    let current_uin = self.uin.expect("Missing uin");
    let users = UserRepository::new(self.app_state.db_pool());
    let friends = contacts.iter()
      .filter(|u| u.user_type != ContactType::Blocked)
      .map(|u| u.uin)
      .collect::<Vec<GGNumber>>();
    let existing_users = users.find_by_uins(&friends).await?
      .iter()
      .map(|u| u.uin)
      .collect::<Vec<GGNumber>>();
    self.contacts.set(contacts);

    tracing::info!("Sending contact list to {}: {} friends, and filtered from: {}", current_uin, existing_users.len(), friends.len());

    let presence_hub = self.app_state.presence_hub();
    presence_hub.subscribe(current_uin, &existing_users);

    let presences = existing_users.iter()
      .map(|uin| self.app_state.presence_hub().find(uin).into())
      .collect::<Vec<ContactStatus>>();

    self.protocol.send(GGPacket::NotifyReply60(presences)).await?;
    tokio::time::sleep(Duration::from_millis(100)).await;
    self.app_state.presence_hub().refresh(current_uin);

    Ok(())
  }

  #[instrument]
  pub async fn cleanup(&mut self) {
    tracing::info!(uin = ?self.uin, "Cleaning up user session");

    self.presence_change_stream = None;
    if let Some(uin) = self.uin {
      self.app_state.message_dispatcher().unregister(uin).await;
      self.app_state.presence_hub().notify(UserPresence::offline(uin));
      self.app_state.presence_hub().unregister(uin, &[uin]); // todo: track contact uids
    }
    let _ =self.protocol.flush().await;
  }
}
