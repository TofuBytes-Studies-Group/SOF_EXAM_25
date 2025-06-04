use std::sync::Arc;
use tokio::sync::Mutex;
// src/models/player.rs
use tokio_postgres::{Client, Error};
use uuid::Uuid;
use crate::dbs::psqldb;
pub(crate) use crate::dbs::psqldb::Database;
use crate::dbs::redisdb::{RedisDatabase};
use crate::dbs::weapondb::{WeaponDB, WeaponDBNoID};

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
#[derive(Debug)]
pub struct PlayerDbNoID {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub defense: i32,
    pub strength: i32,
}

#[derive(Debug)]
pub struct InventoryDb {
    pub id: Uuid,
    pub gold: i32,
}

#[derive(Debug)]
pub struct InventoryDbNoID {
    pub gold: i32,
}
#[derive(Debug)]
pub struct FullPlayerData {
    pub player: PlayerDbNoID,
    pub inventory: InventoryDbNoID,
    pub weapons: Vec<WeaponDBNoID>,
}


impl PlayerDb {
    pub async fn create(
        psql: Arc<Mutex<Client>>,
        redis: Arc<RedisDatabase>,
        name: &str,
        hp: i32,
        max_hp: i32,
        defense: i32,
        strength: i32,
    ) -> Result<PlayerDb, Error> {
        let client = psql.lock().await;
        let row = client.query_one(
            "INSERT INTO player (name, hp, max_hp, defense, strength)
             VALUES ($1, $2, $3, $4, $5)
             RETURNING id, name, hp, max_hp, defense, strength, inventory_id",
            &[&name, &hp, &max_hp, &defense, &strength],
        ).await?;

        let player = PlayerDb {
            id: row.get(0),
            name: row.get(1),
            hp: row.get(2),
            max_hp: row.get(3),
            defense: row.get(4),
            strength: row.get(5),
            inventory_id: row.get(6),
        };

        // players defaut score on creation: 0 kills
        let _ = redis.add_member("scoreboard", &player.name, 0.0).await; // Kills set to 0 on creation as a Placeholder

        Ok(player)
    }

    pub async fn get_player_full_data(
        psql: Arc<Mutex<Client>>,
        player_name: &str,
    ) -> Result<FullPlayerData, Error> {
        let client = psql.lock().await;

        let rows = client
            .query("SELECT * FROM get_player_stats($1)", &[&player_name])
            .await?;

        if rows.is_empty() {
            println!("Player {} not found", player_name);
        }

        let mut weapons = Vec::new();
        let mut player_info: Option<PlayerDbNoID> = None;
        let mut inventory_info: Option<InventoryDbNoID> = None;

        for row in rows {
            // Extract shared player and inventory data once
            if player_info.is_none() {
                player_info = Some(PlayerDbNoID {
                    name: row.get(0),
                    hp: row.get(1),
                    max_hp: row.get(2),
                    defense: row.get(3),
                    strength: row.get(4),
                });
            }

            if inventory_info.is_none() {
                inventory_info = Some(InventoryDbNoID {
                    gold: row.get(5),
                });
            }

            // Weapon data per row
            weapons.push(WeaponDBNoID {
                name: row.get(6),
                damage: row.get(7),
                weight: row.get(8),
                upgrade: row.get(9),
                perk: row.get(10),
                weapon_type: row.get(11),
                predicted_price: Some(row.get(12)),
            });
        }

        Ok(FullPlayerData {
            player: player_info.unwrap(),
            inventory: inventory_info.unwrap(),
            weapons,
        })
    }
}
