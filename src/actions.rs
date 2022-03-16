use neo4rs::Graph;

use crate::transactions::{Input, Transaction};
use crate::users::User;
use crate::utils::{get_address_by_private_key, sign_inputs};

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
   let amount = amount.parse::<f64>().unwrap();
   let private_key = hex::decode(private_key).unwrap();
   let from_address = get_address_by_private_key(&private_key);

   let user = User::new(from_address);
   let (inputs, balance) = user.get_unspend_outputs_as_inputs(graph, amount).await;

   // Verify balance
   verify_balance(balance, amount).unwrap();

   // Verify sign inputs
   let inputs = sign_and_verify_inputs(&private_key, inputs).unwrap();

   println!("{:?}", inputs);
}

fn verify_balance(balance: f64, amount: f64) -> Result<(), String> {
   if balance < amount {
      Err("Insufficient balance".to_string())
   } else {
      Ok(())
   }
}

fn sign_and_verify_inputs(private_key: &[u8], inputs: Vec<Input>) -> Result<Vec<Input>, String> {
   let inputs = sign_inputs(private_key, inputs);
   if !inputs.verify() {
      Err("Some input does not belong to the user".to_string())
   } else {
      Ok(inputs)
   }
}

// Exercise with trait <3

trait Compare {
   fn verify(&self) -> bool;
}

impl Compare for Vec<Input> {
   fn verify(&self) -> bool {
      let mut result = true;
      for input in self {
         if !input.verify() {
            result = false;
            break;
         }
      }
      result
   }
}
