use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
// src/models/player.rs
use tokio_postgres::{Client, Error};
use uuid::Uuid;
use crate::dbs::psqldb;
pub(crate) use crate::dbs::psqldb::Database;
use crate::weapon_prediction::bridge;
use crate::weapon_prediction::bridge::Weapon;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct WeaponDB {
    pub id: Uuid,
    pub name: String,
    pub damage: i32,
    pub weight: f64,
    pub upgrade: String,
    pub perk: String,
    pub weapon_type: String,
    pub predicted_price: Option<f64>,
}

impl WeaponDB {
    pub async fn create_weapon(
        psql: Arc<Mutex<Client>>, name: &str, damage: i32, weight: f64, upgrade: &str, perk: &str, weapon_type: &str, predicted_price: Option<f64> ) -> Result<WeaponDB, Error> {
        let client = psql.lock().await; // lås klienten
        let row = client.query_one(
            "INSERT INTO weapon (name, damage, weight, upgrade, perk, weapon_type, predicted_price) VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING name, damage, weight, upgrade, perk, weapon_type, predicted_price",
            &[&name, &damage, &weight, &upgrade, &perk, &weapon_type, &predicted_price],
        ).await?;

        Ok(WeaponDB {
            id: row.get(0),
            name: row.get(1),
            damage: row.get(2),
            weight: row.get(3),
            upgrade: row.get(4),
            perk: row.get(5),
            weapon_type: row.get(6),
            predicted_price: row.get(7),
        })
    }
}