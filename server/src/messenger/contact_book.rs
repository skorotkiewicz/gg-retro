use gg_protocol::GGNumber;
use gg_protocol::packets::{ContactEntry, ContactType};

pub struct ContactBook {
  buddies: Vec<GGNumber>,
  friends: Vec<GGNumber>,
  blocked: Vec<GGNumber>
}

impl ContactBook {
  pub fn new() -> Self {
    Self {
      buddies: Vec::new(),
      friends: Vec::new(),
      blocked: Vec::new()
    }
  }

  pub fn is_blocked(&self, contact_uin: GGNumber) -> bool {
    return self.blocked.contains(&contact_uin);
  }

  pub fn set(&mut self, contacts: &Vec<ContactEntry>) {
    self.clear();

    for contact in contacts {
      let uin = contact.uin;
      match contact.user_type {
        ContactType::Blocked => {
          self.blocked.push(uin);
        },
        ContactType::Friend => {
          self.friends.push(uin);
        },
        ContactType::Buddy => {
          self.buddies.push(uin);
        }
      }
    }
  }

  pub fn clear(&mut self) {
    self.buddies.clear();
    self.friends.clear();
    self.blocked.clear();
  }
}