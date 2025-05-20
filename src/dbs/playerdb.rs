use std::sync::Arc;
use tokio::sync::Mutex;
// src/models/player.rs
use tokio_postgres::{Client, Error};
use uuid::Uuid;
use crate::dbs::psqldb;
pub(crate) use crate::dbs::psqldb::Database;

#[derive(Debug)]
pub struct PlayerDb {
    pub id: Uuid,
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub defense: i32,
    pub strength: i32,
    pub inventory_id: Option<Uuid>,
}

impl PlayerDb {
    pub async fn create(
        psql: Arc<Mutex<Client>>, name: &str, hp: i32, max_hp: i32, defense: i32, strength: i32, ) -> Result<PlayerDb, Error> {
        let client = psql.lock().await; // lås klienten
        let row = client.query_one(
            "INSERT INTO player (name, hp, max_hp, defense, strength) VALUES ($1, $2, $3, $4, $5) RETURNING id, name, hp, max_hp, defense, strength, inventory_id",
            &[&name, &hp, &max_hp, &defense, &strength],
        ).await?;

        Ok(PlayerDb {
            id: row.get(0),
            name: row.get(1),
            hp: row.get(2),
            max_hp: row.get(3),
            defense: row.get(4),
            strength: row.get(5),
            inventory_id: row.get(6),
        })
    }
}
