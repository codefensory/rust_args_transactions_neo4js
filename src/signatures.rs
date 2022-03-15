use k256::{
   ecdsa::{signature::Verifier, Signature, VerifyingKey},
   elliptic_curve::sec1::ToEncodedPoint,
   SecretKey,
};
use ripemd::{Digest, Ripemd160};
use signature::Signature as Sign;

pub fn _verify_signature(public_key: String, signature: String, message: &[u8]) -> bool {
   let public_key = hex::decode(public_key).unwrap();
   let signature = hex::decode(signature).unwrap();
   let signature = Signature::from_bytes(&signature).unwrap();
   let verify_key = VerifyingKey::from_sec1_bytes(&public_key).unwrap();

   verify_key.verify(&message, &signature).is_ok()
}

pub fn get_address_by_private_key(private_key: &[u8]) -> String {
   let private_key = SecretKey::from_be_bytes(&private_key).unwrap();
   let public_key = private_key.public_key().to_encoded_point(false);

   let mut hasher = Ripemd160::new();
   hasher.update(&public_key.as_bytes());
   hex::encode(hasher.finalize())
}
