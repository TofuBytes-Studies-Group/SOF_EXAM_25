use dotenvy::dotenv;
use std::env;
use std::sync::Arc;
use tokio_postgres::{Client, NoTls, Error};
use bevy::prelude::Resource;
use tokio::sync::Mutex;

#[derive(Resource)]
pub struct Database {
    pub client: Arc<Mutex<Client>>,
}

impl Database {
    pub async fn connect() -> Result<Self, Error> {
        dotenv().ok();
        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        let (client, connection) = tokio_postgres::connect(&db_url, NoTls).await?;

        // Spawn the connection handler
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        Ok(Database { client: Arc::new(Mutex::new(client)) })
    }
}