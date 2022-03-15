use neo4rs::Graph;

use crate::signatures::get_address_by_private_key;
use crate::transactions::Transaction;
use crate::users::User;

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
   let inputs = user.get_unspend_outputs(graph).await;

   println!("------");
   println!("{:?}", inputs);
}
