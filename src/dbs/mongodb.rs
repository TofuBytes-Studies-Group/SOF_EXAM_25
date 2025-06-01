use mongodb::{
    bson::{doc, oid::ObjectId},
    options::ClientOptions,
    Client, Collection, Database,
};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::sync::Arc;
use bevy::prelude::Resource;
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoreEntry {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub character_name: String,
    pub world_lore_lines: Vec<String>,
    pub timestamp: String,
}

#[derive(Resource, Clone)]
pub struct LoreDatabase {
    collection: Arc<Collection<LoreEntry>>,
}

impl LoreDatabase {
    pub async fn new(connection_str: &str) -> Result<Self, Box<dyn Error>> {
        let client_options = ClientOptions::parse(connection_str).await?;
        let client = Client::with_options(client_options)?;
        let db = client.database("mygame");
        let collection = db.collection::<LoreEntry>("lore");
        Ok(LoreDatabase {
            collection: Arc::new(collection),
        })
    }


    pub async fn create_lore_entry(&self, entry: LoreEntry) -> Result<(), mongodb::error::Error> {
        self.collection.insert_one(entry, None).await?;
        Ok(())
    }

    pub async fn read_lore(&self, id: &str) -> Result<Option<LoreEntry>, Box<dyn Error>> {
        let object_id = ObjectId::parse_str(id)?;
        let filter = doc! { "_id": object_id };
        let result = self.collection.find_one(filter, None).await?;
        Ok(result)
    }

    pub async fn update_lore(
        &self,
        id: &str,
        new_lines: Vec<String>,
    ) -> Result<(), Box<dyn Error>> {
        let object_id = ObjectId::parse_str(id)?;
        let filter = doc! { "_id": object_id };
        let update = doc! {
            "$set": {
                "world_lore_lines": new_lines,
                "timestamp": Utc::now().to_rfc3339(),
            }
        };
        self.collection.update_one(filter, update, None).await?;
        Ok(())
    }

    pub async fn delete_lore(&self, id: &str) -> Result<(), Box<dyn Error>> {
        let object_id = ObjectId::parse_str(id)?;
        let filter = doc! { "_id": object_id };
        self.collection.delete_one(filter, None).await?;
        Ok(())
    }
}
