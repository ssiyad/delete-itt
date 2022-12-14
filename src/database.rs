#![allow(dead_code)]
use sqlx::{
    any::{AnyPool, AnyPoolOptions},
    query, query_as, Error, FromRow,
};

use crate::types::VoteType;

#[derive(Debug, Clone)]
pub struct Database {
    pool: AnyPool,
}

#[derive(Debug, Clone, FromRow)]
pub struct Poll {
    pub id: i64,
    pub chat_id: i64,
    pub poll_id: i32,
    pub message_id: i32,
    pub message_user_id: i64,
    pub minimum_vote_count: i64,
    pub vote_count_yes: i64,
    pub vote_count_no: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct Voter {
    pub id: i64,
    pub poll_id: i64,
    pub user_id: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct Chat {
    pub id: i64,
    pub chat_id: i64,
    pub minimum_vote_count: i64,
    pub locale: String,
    pub poll_delete_delay: i64,
}

#[derive(Debug, Clone, FromRow)]
pub struct MessageToDelete {
    pub id: i64,
    pub chat_id: i64,
    pub message_id: i32,
    pub timestamp: i64,
}

static SCHEMA_INIT: &str = "
CREATE TABLE IF NOT EXISTS polls (
id INTEGER PRIMARY KEY,
chat_id INTEGER NOT NULL,
poll_id INTEGER NOT NULL,
message_id INTEGER NOT NULL,
message_user_id INTEGER NOT NULL,
minimum_vote_count INTEGER NOT NULL,
vote_count_yes INTEGER DEFAULT 0,
vote_count_no INTEGER DEFAULT 0
);

CREATE TABLE IF NOT EXISTS voters (
id INTEGER PRIMARY KEY,
poll_id INTEGER NOT NULL,
user_id INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS chats (
id INTEGER PRIMARY KEY,
chat_id INTEGER NOT NULL,
minimum_vote_count INTEGER DEFAULT 5,
locale VARCHAR DEFAULT 'en',
poll_delete_delay INTEGER DEFAULT 5
);

CREATE TABLE IF NOT EXISTS scheduled_to_delete (
id INTEGER PRIMARY KEY,
chat_id INTEGER NOT NULL,
message_id INTEGER NOT NULL,
timestamp INTEGER NOT NULL
);
";

impl Database {
    pub async fn new<S>(url: S) -> Self
    where
        S: Into<String>,
    {
        let pool = AnyPoolOptions::new()
            .max_connections(5)
            .connect(&url.into())
            .await
            .expect("Database connection failed");

        let db = Database { pool };

        db.init().await;

        db
    }

    async fn init(&self) {
        sqlx::query(SCHEMA_INIT)
            .execute(&self.pool)
            .await
            .expect("Database initialisation failed");
    }

    pub async fn create_poll(
        &self,
        chat_id: i64,
        poll_id: i32,
        message_id: i32,
        message_user_id: i64,
        minimum_vote_count: i64,
    ) -> Result<(), Error> {
        query(
            "INSERT INTO polls \
            (chat_id, poll_id, message_id, message_user_id, minimum_vote_count) \
            VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(chat_id)
        .bind(poll_id)
        .bind(message_id)
        .bind(message_user_id)
        .bind(minimum_vote_count)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn get_poll(&self, chat_id: i64, poll_id: i32) -> Result<Option<Poll>, Error> {
        query_as::<_, Poll>("SELECT * FROM polls WHERE chat_id = $1 AND poll_id = $2")
            .bind(chat_id)
            .bind(poll_id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn register_vote(&self, poll_id: i64, v: VoteType) -> Result<bool, Error> {
        match v {
            VoteType::Yes => {
                let affected =
                    query("UPDATE polls SET vote_count_yes = vote_count_yes + 1 WHERE id = $1")
                        .bind(poll_id)
                        .execute(&self.pool)
                        .await?
                        .rows_affected();

                Ok(affected > 0)
            }
            VoteType::No => {
                let affected =
                    query("UPDATE polls SET vote_count_no = vote_count_no + 1 WHERE id = $1")
                        .bind(poll_id)
                        .execute(&self.pool)
                        .await?
                        .rows_affected();

                Ok(affected > 0)
            }
        }
    }

    pub async fn remove_poll(&self, poll_id: i64) -> Result<bool, Error> {
        let affected = query("DELETE FROM polls WHERE id = $1")
            .bind(poll_id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(affected > 0)
    }

    pub async fn create_voter(&self, poll_id: i64, user_id: i64) -> Result<(), Error> {
        query("INSERT INTO voters (poll_id, user_id) VALUES ($1, $2)")
            .bind(poll_id)
            .bind(user_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_voter(&self, poll_id: i64, user_id: i64) -> Result<Option<Voter>, Error> {
        query_as::<_, Voter>("SELECT * FROM voters WHERE poll_id = $1 AND user_id = $2")
            .bind(poll_id)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn remove_voter(&self, voter_id: i64) -> Result<bool, Error> {
        let affected = query("DELETE FROM voters WHERE id = $1")
            .bind(voter_id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(affected > 0)
    }

    pub async fn remove_voters(&self, poll_id: i64) -> Result<bool, Error> {
        let affected = query("DELETE FROM voters WHERE poll_id = $1")
            .bind(poll_id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(affected > 0)
    }

    pub async fn create_chat(&self, chat_id: i64) -> Result<bool, Error> {
        let affected = query("INSERT INTO chats (chat_id) VALUES ($1)")
            .bind(chat_id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(affected > 0)
    }

    pub async fn get_chat(&self, chat_id: i64) -> Result<Option<Chat>, Error> {
        query_as::<_, Chat>("SELECT * FROM chats WHERE chat_id = $1")
            .bind(chat_id)
            .fetch_optional(&self.pool)
            .await
    }

    pub async fn get_chat_votes(&self, chat_id: i64) -> Result<Option<i64>, Error> {
        let x = query_as::<_, (i64,)>("SELECT minimum_vote_count FROM chats WHERE chat_id = $1")
            .bind(chat_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some((y,)) = x {
            Ok(Some(y))
        } else {
            Ok(None)
        }
    }

    pub async fn set_chat_votes(&self, chat_id: i64, votes_count: i64) -> Result<bool, Error> {
        let affected = query("UPDATE chats SET minimum_vote_count = $1 WHERE chat_id = $2")
            .bind(votes_count)
            .bind(chat_id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(affected > 0)
    }

    pub async fn get_chat_locale(&self, chat_id: i64) -> Result<Option<String>, Error> {
        let x = query_as::<_, (String,)>("SELECT locale FROM chats WHERE chat_id = $1")
            .bind(chat_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some((y,)) = x {
            Ok(Some(y))
        } else {
            Ok(None)
        }
    }

    pub async fn set_chat_locale(&self, chat_id: i64, locale: &str) -> Result<bool, Error> {
        let affected = query("UPDATE chats SET locale = $1 WHERE chat_id = $2")
            .bind(locale)
            .bind(chat_id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(affected > 0)
    }

    pub async fn get_chat_poll_delete_delay(&self, chat_id: i64) -> Result<Option<i64>, Error> {
        let x = query_as::<_, (i64,)>("SELECT poll_delete_delay FROM chats WHERE chat_id = $1")
            .bind(chat_id)
            .fetch_optional(&self.pool)
            .await?;

        if let Some((y,)) = x {
            Ok(Some(y))
        } else {
            Ok(None)
        }
    }

    pub async fn set_chat_poll_delete_delay(
        &self,
        chat_id: i64,
        poll_delete_delay: i64,
    ) -> Result<bool, Error> {
        let affected = query("UPDATE chats SET poll_delete_delay = $1 WHERE chat_id = $2")
            .bind(poll_delete_delay)
            .bind(chat_id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(affected > 0)
    }

    pub async fn remove_chat(&self, chat_id: i64) -> Result<bool, Error> {
        let affected = query("DELETE FROM chats WHERE chat_id = $1")
            .bind(chat_id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(affected > 0)
    }

    pub async fn schedule_message_delete(
        &self,
        chat_id: i64,
        message_id: i64,
        timestamp: i64,
    ) -> Result<bool, Error> {
        let affected = query(
            "INSERT INTO scheduled_to_delete (chat_id, message_id, timestamp) VALUES ($1, $2, $3)",
        )
        .bind(chat_id)
        .bind(message_id)
        .bind(timestamp)
        .execute(&self.pool)
        .await?
        .rows_affected();

        Ok(affected > 0)
    }

    pub async fn get_pending_messages_to_delete(
        &self,
        timestamp: i64,
    ) -> Result<Vec<MessageToDelete>, Error> {
        query_as("SELECT * FROM scheduled_to_delete WHERE timestamp <= $1")
            .bind(timestamp)
            .fetch_all(&self.pool)
            .await
    }

    pub async fn remove_from_scheduled_delete(&self, id: i64) -> Result<bool, Error> {
        let affected = query("DELETE FROM scheduled_to_delete WHERE id = $1")
            .bind(id)
            .execute(&self.pool)
            .await?
            .rows_affected();

        Ok(affected > 0)
    }
}
