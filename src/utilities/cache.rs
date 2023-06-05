use std::collections::HashMap;
use std::fmt;

use tdlib::enums::{ChatMemberStatus, ChatType, MessageSender, UserType};
use tdlib::types::{
    Chat, ChatPermissions, MessageSenderUser, UpdateChatMember, UpdateChatPermissions,
    UpdateChatTitle, UpdateNewChat, UpdateUser, User,
};

use super::telegram_utils::MainUsername;

#[derive(Default)]
pub struct Cache {
    chats: HashMap<i64, CompactChat>,
    users: HashMap<i64, CompactUser>,
    member_status: HashMap<(i64, i64), ChatMemberStatus>,
}

impl Cache {
    pub fn get_chat(&self, id: i64) -> Option<CompactChat> {
        self.chats.get(&id).cloned()
    }

    pub fn get_user(&self, id: i64) -> Option<CompactUser> {
        self.users.get(&id).cloned()
    }

    pub fn get_member_status(&self, chat_id: i64, member_id: i64) -> Option<ChatMemberStatus> {
        self.member_status.get(&(chat_id, member_id)).cloned()
    }

    pub fn set_member_status(&mut self, chat_id: i64, member_id: i64, status: ChatMemberStatus) {
        self.member_status.insert((chat_id, member_id), status);
    }

    pub fn update_new_chat(&mut self, update: UpdateNewChat) {
        self.chats.insert(update.chat.id, update.chat.into());
    }

    pub fn update_chat_title(&mut self, update: UpdateChatTitle) {
        if let Some(chat) = self.chats.get_mut(&update.chat_id) {
            chat.title = update.title;
        }
    }

    pub fn update_chat_permissions(&mut self, update: UpdateChatPermissions) {
        if let Some(chat) = self.chats.get_mut(&update.chat_id) {
            chat.permissions = update.permissions;
        }
    }

    pub fn update_user(&mut self, update: UpdateUser) {
        self.users.insert(update.user.id, update.user.into());
    }

    pub fn update_chat_member(&mut self, update: UpdateChatMember) {
        if let MessageSender::User(MessageSenderUser { user_id }) = update.new_chat_member.member_id
        {
            self.member_status.insert((update.chat_id, user_id), update.new_chat_member.status);
        }
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
        let username = value.main_username().map(Into::into);

        Self {
            id: value.id,
            first_name: value.first_name,
            last_name: value.last_name,
            username,
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
