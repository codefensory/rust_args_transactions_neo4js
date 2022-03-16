use k256::{
   ecdsa::{signature::Signer, Signature, SigningKey},
   elliptic_curve::sec1::ToEncodedPoint,
   SecretKey,
};
use ripemd::{Digest, Ripemd160};

use crate::transactions::Input;

pub fn get_address_by_private_key(private_key: &[u8]) -> String {
   let private_key = SecretKey::from_be_bytes(&private_key).unwrap();
   let public_key = private_key.public_key().to_encoded_point(false);

   let mut hasher = Ripemd160::new();
   hasher.update(&public_key.as_bytes());
   hex::encode(hasher.finalize())
}

pub fn sign_inputs(private_key: &[u8], inputs: Vec<Input>) -> Vec<Input> {
   let signkey = SigningKey::from_bytes(private_key).unwrap();
   let public_key = hex::encode(signkey.verifying_key().to_encoded_point(false));
   let mut result = Vec::new();

   for input in inputs {
      let signature: Signature = signkey.sign(&input.prev_tx.as_bytes());
      let mut new_input = input.clone();
      new_input.signature = hex::encode(signature);
      new_input.public_key = public_key.clone();
      result.push(new_input);
   }

   result
}

pub fn to_float_string(value: f32) -> String {
   if value % 1.0 == 0.0 {
      format!("{:.1}", value)
   } else {
      format!("{}", value)
   }
}
