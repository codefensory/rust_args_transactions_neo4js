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
   if balance < amount {
      println!("Insufficient balance");
      return;
   }

   let inputs = sign_inputs(&private_key, inputs);

   println!("{:?}", inputs);
}