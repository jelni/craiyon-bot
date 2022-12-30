use std::collections::HashMap;
use std::fmt;

use tdlib::enums::{ChatType, Update, UserType};
use tdlib::types::{Chat, ChatPermissions, UpdateNewChat, UpdateUser, User};

#[derive(Default)]
pub struct Cache {
    chats: HashMap<i64, CompactChat>,
    users: HashMap<i64, CompactUser>,
}

impl Cache {
    pub fn get_chat(&self, id: i64) -> Option<CompactChat> {
        self.chats.get(&id).cloned()
    }

    pub fn get_user(&self, id: i64) -> Option<CompactUser> {
        self.users.get(&id).cloned()
    }

    pub fn update(&mut self, update: Update) {
        match update {
            Update::NewChat(UpdateNewChat { chat }) => {
                self.chats.insert(chat.id, chat.into());
            }
            Update::ChatTitle(update) => {
                if let Some(chat) = self.chats.get_mut(&update.chat_id) {
                    chat.title = update.title;
                }
            }
            Update::ChatPermissions(update) => {
                if let Some(chat) = self.chats.get_mut(&update.chat_id) {
                    chat.permissions = update.permissions;
                }
            }
            Update::User(UpdateUser { user }) => {
                self.users.insert(user.id, user.into());
            }
            _ => (),
        };
    }
}

#[derive(Clone)]
pub struct CompactChat {
    pub r#type: ChatType,
    pub title: String,
    pub permissions: ChatPermissions,
}

impl From<Chat> for CompactChat {
    fn from(value: Chat) -> Self {
        Self { r#type: value.r#type, title: value.title, permissions: value.permissions }
    }
}

impl fmt::Display for CompactChat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.r#type {
            ChatType::Private(_) => write!(f, "PM"),
            _ => write!(f, "{:?}", self.title),
        }
    }
}

#[derive(Clone)]
pub struct CompactUser {
    pub id: i64,
    pub first_name: String,
    pub last_name: String,
    pub username: Option<String>,
    pub r#type: UserType,
    pub language_code: String,
}

impl From<User> for CompactUser {
    fn from(value: User) -> Self {
        Self {
            id: value.id,
            first_name: value.first_name,
            last_name: value.last_name,
            username: value
                .usernames
                .and_then(|usernames| usernames.active_usernames.into_iter().next()),
            r#type: value.r#type,
            language_code: value.language_code,
        }
    }
}

impl fmt::Display for CompactUser {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(username) = &self.username {
            write!(f, "@{username}")?;
        } else {
            write!(f, "{}", self.first_name)?;
            if !self.last_name.is_empty() {
                write!(f, " {}", self.last_name)?;
            }
        }

        Ok(())
    }
}
