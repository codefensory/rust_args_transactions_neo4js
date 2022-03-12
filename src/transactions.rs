use base58::ToBase58;
use chrono::Utc;
use neo4rs::Graph;
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Output {
   id: u32,
   value: f64,
   address: String,
}

pub struct Input {
   prev_tx: String,
   id: u32,
   value: f64,

   address: String,
   public_key: String,
   signature: String,
}

#[derive(Debug, Serialize)]
pub struct Transaction {
   hash: String,
   uuid: String,
   date: i64,
   vin_hash: String,
   vout_hash: String,

   #[serde(skip_serializing)]
   vout: Vec<Output>,
}

impl Transaction {
   pub fn new() -> Transaction {
      Transaction {
         hash: String::new(),
         uuid: Uuid::new_v4().as_bytes().to_base58(),
         date: Utc::now().timestamp(),
         vin_hash: String::new(),
         vout_hash: String::new(),
         vout: Vec::new(),
      }
   }

   pub fn generate_hash(&mut self) {
      let vout_json = serde_json::to_string(&self.vout).unwrap();

      let mut vout_hasher = Sha256::new();
      vout_hasher.update(&vout_json);
      self.vout_hash = hex::encode(vout_hasher.finalize());

      let mut self_hasher = Sha256::new();
      let self_json = serde_json::to_string(&self).unwrap();
      self_hasher.update(&self_json);
      self.hash = hex::encode(self_hasher.finalize());
   }

   pub fn add_output(&mut self, amount: f64, to_address: String) {
      let output = Output {
         id: self.vout.len() as u32,
         value: amount,
         address: to_address,
      };

      self.vout.push(output);
   }
}

pub fn create_coinbase(to_address: String, amount: String, _graph: Graph) {
   let mut transaction = Transaction::new();

   transaction.add_output(amount.parse::<f64>().unwrap(), to_address);

   transaction.generate_hash();
   println!("{:?}", transaction)
}
