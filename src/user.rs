use crate::{db::Database, error::CardsError, types::UserId};
use rusqlite::{OptionalExtension, ToSql};
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize)]
pub struct User {
    pub id: UserId,
    pub name: String,
    #[serde(skip)]
    pub realm: String,
    #[serde(skip)]
    pub external_id: String,
}

#[derive(Debug)]
pub struct ExternalUser {
    pub name: String,
    pub realm: String,
    pub external_id: String,
}

struct Cache {
    auth_tokens: HashMap<String, UserId>,
    users: HashMap<UserId, User>,
}

impl Cache {
    fn new() -> Self {
        Self {
            auth_tokens: HashMap::new(),
            users: HashMap::new(),
        }
    }

    fn get_user_id(&self, auth_token: &str) -> Option<UserId> {
        self.auth_tokens.get(auth_token).cloned()
    }

    fn get_user(&self, id: UserId) -> Option<&User> {
        self.users.get(&id)
    }

    fn insert_with_token(&mut self, auth_token: String, user: User) {
        self.auth_tokens.insert(auth_token, user.id);
        self.insert(user);
    }

    fn insert(&mut self, user: User) {
        self.users.insert(user.id, user);
    }
}

#[derive(Clone)]
pub struct Users {
    db: Database,
    cache: Arc<Mutex<Cache>>,
}

impl Users {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            cache: Arc::new(Mutex::new(Cache::new())),
        }
    }

    pub async fn get_user_id(&self, auth_token: String) -> Result<UserId, CardsError> {
        let cache = self.cache.lock().await;
        if let Some(user_id) = cache.get_user_id(&auth_token) {
            return Ok(user_id);
        }
        drop(cache);
        match self.db.run_read_only(|tx| {
            tx.query_row(
                "SELECT user.user_id, user.name, user.realm, user.external_id FROM auth_token, user
                    WHERE auth_token.token = ? AND auth_token.user_id = user.user_id",
                &[&auth_token],
                |row| {
                    Ok(User {
                        id: row.get_unwrap::<_, UserId>(0),
                        name: row.get_unwrap::<_, String>(1),
                        realm: row.get_unwrap::<_, String>(2),
                        external_id: row.get_unwrap::<_, String>(3),
                    })
                },
            )
            .optional()
            .map_err(|e| e.into())
        }) {
            Ok(user) => match user {
                Some(user) => {
                    let user_id = user.id;
                    let mut cache = self.cache.lock().await;
                    cache.insert_with_token(auth_token, user);
                    Ok(user_id)
                }
                None => Err(CardsError::UnknownAuthToken(auth_token)),
            },
            Err(e) => Err(e),
        }
    }

    pub async fn get_users(&self, mut ids: Vec<UserId>) -> Result<HashSet<User>, CardsError> {
        let cache = self.cache.lock().await;
        let mut cached = HashSet::new();
        ids.retain(|id| {
            if let Some(user) = cache.get_user(*id) {
                cached.insert(user.clone());
                false
            } else {
                true
            }
        });
        drop(cache);
        if ids.is_empty() {
            return Ok(cached);
        }

        let mut uncached = HashSet::new();
        self.db.run_read_only(|tx| {
            let mut stmt =
                tx.prepare("SELECT name, realm, external_id FROM user WHERE user_id = ?")?;
            for id in &ids {
                let user = stmt
                    .query_row(&[id], |row| {
                        Ok(User {
                            id: *id,
                            name: row.get_unwrap::<_, String>(0),
                            realm: row.get_unwrap::<_, String>(1),
                            external_id: row.get_unwrap::<_, String>(2),
                        })
                    })
                    .optional()?;
                if let Some(user) = user {
                    uncached.insert(user);
                }
            }
            Ok(())
        })?;
        let mut cache = self.cache.lock().await;
        for user in &uncached {
            cache.insert(user.clone());
        }
        cached.extend(uncached);
        Ok(cached)
    }

    pub async fn insert(&self, auth_token: String, user: ExternalUser) -> Result<User, CardsError> {
        let id = self.db.run_with_retry(|tx| {
            let mut id = UserId::new();
            let edits = tx.execute::<&[&dyn ToSql]>(
                "INSERT INTO user (user_id, name, realm, external_id)
                    VALUES (?, ?, ?, ?) ON CONFLICT DO NOTHING",
                &[&id, &user.name, &user.realm, &user.external_id],
            )?;
            if edits == 0 {
                id = tx.query_row_and_then(
                    "SELECT user_id FROM user WHERE realm = ? and external_id = ?",
                    &[&user.realm, &user.external_id],
                    |row| row.get::<_, UserId>(0),
                )?;
            }
            tx.execute::<&[&dyn ToSql]>(
                "INSERT INTO auth_token (token, user_id)
                    VALUES (?, ?) ON CONFLICT DO NOTHING",
                &[&auth_token, &id],
            )?;
            Ok(id)
        })?;
        let mut cache = self.cache.lock().await;
        let ExternalUser {
            name,
            realm,
            external_id,
        } = user;
        let user = User {
            id,
            name,
            realm,
            external_id,
        };
        cache.insert_with_token(auth_token, user.clone());
        Ok(user)
    }
}
