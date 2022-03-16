use base58::ToBase58;
use chrono::Utc;
use serde::Serialize;
use sha2::{Digest, Sha256};
use ripemd::Ripemd160;
use uuid::Uuid;

use neo4rs::{query, Graph, Node};
use k256:: {
   ecdsa:: {
      signature::Verifier,
      Signature, VerifyingKey,
   },
};
use signature::Signature as Sign;

#[derive(Debug, Serialize, Clone)]
pub struct Transaction {
   #[serde(skip_serializing)]
   pub hash: String,

   uuid: String,
   date: i64,
   vin_hash: String,
   vout_hash: String,

   #[serde(skip_serializing)]
   pub vout: Vec<Output>,
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
}

impl Transaction {
   pub fn from_node(node: &Node, outputs: Vec<Node>, inputs: Vec<Node>) -> Self {
      let mut vout: Vec<Output> = outputs.iter().map(Output::from_node).collect();
      vout.sort_by(|a, b| b.id.cmp(&a.id));

      let mut _vin: Vec<Input> = inputs.iter().map(Input::from_node).collect();
      _vin.sort_by(|a, b| b.id.cmp(&a.id));

      Transaction {
         hash: node.get("hash").unwrap_or(String::new()),
         uuid: node.get("uuid").unwrap_or(String::new()),
         date: node.get("date").unwrap_or(0),
         vin_hash: node.get("vin_hash").unwrap_or(String::new()),
         vout_hash: node.get("vout_hash").unwrap_or(String::new()),
         vout,
         _vin,
      }
   }

   pub fn validate(&mut self) -> bool {
      let prev_hash = self.hash.clone();
      self.generate_hash();
      prev_hash == self.hash
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

      let json = serde_json::to_string(&self).unwrap();
      let mut self_hasher = Sha256::new();
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
            &vout.address
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

// INPUTS AND OUTPUTS

//------
// Outputs
//------
#[derive(Debug, Serialize, Clone)]
pub struct Output {
   pub id: u32,
   pub value: f64,
   pub address: String,
}

impl Output {
   pub fn from_node(node: &Node) -> Self {
      Output {
         id: node.get::<i64>("id").unwrap() as u32,
         value: node.get::<f64>("value").unwrap(),
         address: node.get("address").unwrap(),
      }
   }
}

//------
// Inputs
//------
#[derive(Debug, Serialize, Clone)]
pub struct Input {
   pub prev_tx: String,
   pub id: u32,
   pub value: f64,

   pub address: String,
   pub public_key: String,
   pub signature: String,
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

   pub fn from_node(node: &Node) -> Self {
      Input {
         prev_tx: node.get("prev_tx").unwrap_or(String::new()),
         id: node.get::<i64>("id").unwrap_or(0) as u32,
         value: node.get("value").unwrap_or(0.0),
         address: node.get("address").unwrap_or(String::new()),
         public_key: node.get("public_key").unwrap_or(String::new()),
         signature: node.get("signature").unwrap_or(String::new()),
      }
   }

   pub fn verify(&self) -> bool {
      let public_key = hex::decode(&self.public_key).unwrap();

      // Verifi address from public_key
      let mut pk_hasher = Ripemd160::new();
      pk_hasher.update(&public_key);
      let address = hex::encode(pk_hasher.finalize());
      if address != self.address {
         return false
      }

      let signature = hex::decode(&self.signature).unwrap();
      let signature = Signature::from_bytes(&signature).unwrap();
      let verify_key = VerifyingKey::from_sec1_bytes(&public_key).unwrap();
      
      verify_key.verify(&self.prev_tx.as_bytes(), &signature).is_ok()
   }
}
