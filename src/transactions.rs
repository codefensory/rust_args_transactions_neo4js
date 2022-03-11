use base58::ToBase58;
use chrono::Utc;
use neo4rs::Graph;
use uuid::Uuid;
use serde::Serialize;
use sha2::{Sha256, Digest};

#[derive(Debug, Serialize)]
pub struct Transaction {
   hash: String,
   uuid: String,
   date: i64,
   vin_hash: String,
   vout_hash: String,
}

impl Transaction {
   pub fn new() -> Transaction {
      Transaction {
         hash: String::new(),
         uuid: Uuid::new_v4().as_bytes().to_base58(),
         date: Utc::now().timestamp(),
         vin_hash: String::new(),
         vout_hash: String::new(),
      }
   }

   pub fn generate_hash(&mut self) {
      let self_json = serde_json::to_string(&self).unwrap();
      let mut hasher = Sha256::new();
      hasher.update(&self_json);
      self.hash = hex::encode(hasher.finalize());
   }
}

pub fn create_coinbase(_to_address: String, _amount: String, _graph: Graph) {
   let mut transaction = Transaction::new();
   transaction.generate_hash();
   println!("{:?}", transaction)
}
