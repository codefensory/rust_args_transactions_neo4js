use base58::ToBase58;
use chrono::Utc;
use neo4rs::query;
use neo4rs::{Graph, Node};
use serde::Serialize;
use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::signatures::get_address_by_private_key;

pub struct User {
   address: String,
}

impl User {
   pub fn new(address: String) -> Self {
      User { address }
   }

   pub async fn get_unspend_outups(&self, graph: Graph) -> Vec<(Transaction, u32)> {
      let mut response = graph
         .execute(
            query(
               r#"
         MATCH (:User {address: $address})-[:OWN]->(o:Output)<-[:OUT]-(tx:Transaction)
         WHERE isEmpty((o)-[:IN]->(:Transaction))
         WITH o.id as oid, tx
         WITH oid, tx, [(tx)-[:OUT]->(o:Output) | o] as t_outputs
         WITH oid, tx, t_outputs, [(tx)-[:IN]->(i:Output) | i] as t_inputs
         RETURN oid, tx, t_outputs, t_inputs
         "#,
            )
            .param("address", &*self.address),
         )
         .await
         .unwrap();

      let mut outputs: Vec<(Transaction, u32)> = Vec::new();

      while let Ok(Some(row)) = response.next().await {
         //let out_id = row.get("out_id").unwrap();
         let output: Vec<Node> = row.get("t_outputs").unwrap();
         println!("{:?}", output);
         //outputs.push((prev_tx, output));
      }

      outputs
   }
}

#[derive(Debug, Serialize, Clone)]
pub struct Output {
   id: u32,
   value: f64,
   address: String,
}

impl Output {
   pub fn from_node(node: Node) -> Self {
      Output {
         id: node.get::<i64>("id").unwrap() as u32,
         value: node.get::<String>("value").unwrap().parse::<f64>().unwrap(),
         address: node.get("address").unwrap(),
      }
   }
}

#[derive(Debug, Serialize, Clone)]
pub struct Input {
   prev_tx: String,
   id: u32,
   value: f64,

   address: String,
   public_key: String,
   signature: String,
}

impl Input {
   pub fn new(prev_tx: String, output: Output) -> Self {
      Input {
         prev_tx,
         id: output.id,
         value: output.value,
         address: output.address,

         public_key: String::new(),
         signature: String::new(),
      }
   }

   pub fn verify(&self) -> bool {
      false
   }
}

#[derive(Debug, Serialize, Clone)]
pub struct Transaction {
   #[serde(skip_serializing)]
   hash: String,

   uuid: String,
   date: i64,
   vin_hash: String,
   vout_hash: String,

   #[serde(skip_serializing)]
   vout: Vec<Output>,
   #[serde(skip_serializing)]
   _vin: Vec<Input>,
}

impl Transaction {
   pub fn new() -> Self {
      Transaction {
         hash: String::new(),
         uuid: Uuid::new_v4().as_bytes().to_base58(),
         date: Utc::now().timestamp(),
         vin_hash: String::new(),
         vout_hash: String::new(),
         vout: Vec::new(),
         _vin: Vec::new(),
      }
   }

   pub fn from_node(node: Node) -> Self {
      Transaction {
         hash: String::new(),
         uuid: node.get("uuid").unwrap(),
         date: node.get("date").unwrap(),
         vin_hash: node.get("vin_hash").unwrap(),
         vout_hash: node.get("vout_hash").unwrap(),
         vout: Vec::new(),
         _vin: Vec::new(),
      }
   }

   pub fn generate_hash(&mut self) {
      let json = serde_json::to_string(&self.vout).unwrap();

      let mut vout_hasher = Sha256::new();
      vout_hasher.update(&json);
      self.vout_hash = hex::encode(vout_hasher.finalize());

      let json = serde_json::to_string(&self._vin).unwrap();
      let mut vin_hasher = Sha256::new();
      vin_hasher.update(&json);
      self.vin_hash = hex::encode(vin_hasher.finalize());

      let mut self_hasher = Sha256::new();
      let json = serde_json::to_string(&self).unwrap();
      self_hasher.update(&json);
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

   pub async fn upload(&mut self, graph: Graph) {
      self.generate_hash();

      let mut queries = Vec::new();

      // Create transaction
      queries.push(format!(
         "CREATE (tx:Transaction {{ hash: '{}', uuid: '{}', date: {}, vin_hash: '{}', vout_hash: '{}' }})",
         &*self.hash, &*self.uuid, self.date, &*self.vin_hash, &*self.vout_hash,
      ));

      for (index, vout) in self.vout.clone().into_iter().enumerate() {
         let user = "uo".to_owned() + &index.to_string();

         queries.push(format!(
            "MERGE ({}:User {{address: '{}'}})",
            user,
            vout.address.clone()
         ));

         let value = if vout.value % 1.0 == 0.0 {
            format!("{:.1}", vout.value)
         } else {
            format!("{}", vout.value)
         };

         queries.push(format!(
            "CREATE (tx)-[:OUT]->(:Output {{id: {}, value: {}, address: '{}'}})<-[:OWN]-({})",
            vout.id, value, vout.address, user
         ));
      }

      graph.run(query(queries.join("\n").as_str())).await.unwrap();
   }
}

pub async fn create_coinbase(to_address: String, amount: String, graph: Graph) {
   let mut transaction = Transaction::new();

   transaction.add_output(amount.parse::<f64>().unwrap(), to_address);
   transaction.upload(graph).await;
}

pub async fn send_transaction(
   private_key: String,
   to_address: String,
   amount: String,
   graph: Graph,
) {
   let private_key = hex::decode(private_key).unwrap();
   let from_address = get_address_by_private_key(&private_key);

   let user = User::new(from_address);
   let inputs = user.get_unspend_outups(graph).await;
   println!("{:?}", inputs);
}
