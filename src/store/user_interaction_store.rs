use async_trait::async_trait;
use std::collections::HashMap;
use tokio::sync::RwLock;
use sqlx::{PgPool, Row};
use std::sync::Arc;

use crate::models::UserInteraction;

#[async_trait]
pub trait UserInteractionStore {
    async fn get_user_interaction(&self, user_id: &str) -> UserInteraction;
    async fn increment_positive_interaction(&self, user_id: &str);
    async fn increment_negative_interaction(&self, user_id: &str);
}

pub struct InMemoryUserInteractionStore {
    store: Arc<RwLock<HashMap<String, UserInteraction>>>,
}

impl InMemoryUserInteractionStore {
    pub fn new() -> Self {
        InMemoryUserInteractionStore {
            store: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[async_trait]
impl UserInteractionStore for InMemoryUserInteractionStore {
    async fn get_user_interaction(&self, user_id: &str) -> UserInteraction {
        let store = self.store.read().await;
        store.get(user_id).cloned().unwrap_or_default()
    }

    async fn increment_positive_interaction(&self, user_id: &str) {
        let mut store = self.store.write().await;
        let interaction = store.entry(user_id.to_string()).or_insert(UserInteraction::default());
        interaction.num_positive += 1;
    }

    async fn increment_negative_interaction(&self, user_id: &str) {
        let mut store = self.store.write().await;
        let interaction = store.entry(user_id.to_string()).or_insert(UserInteraction::default());
        interaction.num_negative += 1;
    }
}

pub struct PostgresUserInteractionStore {
    pool: PgPool,
}

impl PostgresUserInteractionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserInteractionStore for PostgresUserInteractionStore {
    async fn get_user_interaction(&self, user_id: &str) -> UserInteraction {
        let query = "SELECT num_positive, num_negative FROM user_interaction WHERE user_id = $1";
        let row = sqlx::query(query)
            .bind(user_id)
            .fetch_optional(&self.pool)
            .await
            .expect("Failed to fetch user interaction");

        let user_interaction = row
                    .map(|r| UserInteraction {
                        num_positive: r.get::<i32, _>("num_positive") as usize,
                        num_negative: r.get::<i32, _>("num_negative") as usize,
                    })
                    .unwrap_or_default();
        user_interaction
    }
    
    async fn increment_positive_interaction(&self, user_id: &str) {
        let query = r#"
            INSERT INTO user_interaction (user_id, num_positive, num_negative)
            VALUES ($1, 1, 0)
            ON CONFLICT (user_id) 
            DO UPDATE SET 
                num_positive = user_interaction.num_positive + 1,
                last_interaction = CURRENT_TIMESTAMP
        "#;
        sqlx::query(query)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .expect("Failed to increment positive interaction");
    }

    async fn increment_negative_interaction(&self, user_id: &str) {
        let query = r#"
            INSERT INTO user_interaction (user_id, num_positive, num_negative)
            VALUES ($1, 0, 1)
            ON CONFLICT (user_id) 
            DO UPDATE SET
                num_negative = user_interaction.num_negative + 1,
                last_interaction = CURRENT_TIMESTAMP
        "#;
        sqlx::query(query)
            .bind(user_id)
            .execute(&self.pool)
            .await
            .expect("Failed to increment negative interaction");
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_in_memory_user_interaction_store() {
        let store = InMemoryUserInteractionStore::new();
        let user_id = "test_user";

        // Check that the user_id isn't in the store initially
        let user_interaction = store.get_user_interaction(user_id).await;
        assert!(user_interaction.num_positive == 0 && user_interaction.num_negative == 0);

        // Increment positive interaction
        store.increment_positive_interaction(user_id).await;
        let user_interaction = store.get_user_interaction(user_id).await;
        assert_eq!(user_interaction.num_positive, 1);
        assert_eq!(user_interaction.num_negative, 0);

        // Increment negative interaction
        store.increment_negative_interaction(user_id).await;
        let user_interaction = store.get_user_interaction(user_id).await;
        assert_eq!(user_interaction.num_positive, 1);
        assert_eq!(user_interaction.num_negative, 1);
    }

    #[tokio::test]
    async fn test_postgres_user_interaction_store() {
        let pool = PgPool::connect("postgres://username:password@localhost/toodles")
            .await
            .expect("Failed to connect to database");
        let store = PostgresUserInteractionStore::new(pool);
        let user_id = "test_user_2";

        // Check that the user_id isn't in the store initially
        let user_interaction = store.get_user_interaction(user_id).await;
        assert!(user_interaction.num_positive == 0 && user_interaction.num_negative == 0);

        // Increment positive interaction
        store.increment_positive_interaction(user_id).await;
        let user_interaction = store.get_user_interaction(user_id).await;
        assert_eq!(user_interaction.num_positive, 1);
        assert_eq!(user_interaction.num_negative, 0);

        // Increment negative interaction
        store.increment_negative_interaction(user_id).await;
        let user_interaction = store.get_user_interaction(user_id).await;
        assert_eq!(user_interaction.num_positive, 1);
        assert_eq!(user_interaction.num_negative, 1);
    }
}
