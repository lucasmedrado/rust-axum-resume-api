use std::sync::Arc;

use sqlx::PgPool;

use crate::config::Config;

/// Estado compartilhado entre todas as requisições.
/// `Clone` é barato: `PgPool` e `Arc` apenas incrementam um contador.
#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub config: Arc<Config>,
}

impl AppState {
    pub fn new(pool: PgPool, config: Config) -> Self {
        Self {
            pool,
            config: Arc::new(config),
        }
    }
}
