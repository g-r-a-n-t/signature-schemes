extern crate amcl;

use super::amcl_utils::{hash_on_GroupG1, ate_pairing};
use super::types::GroupG1;
use super::constants::GeneratorG2;
use super::keys::{
    SecretKey,
    PublicKey,
};
use bls::errors::SerzDeserzError;
use bls::amcl_utils::get_G1_point_from_bytes;
use bls::amcl_utils::get_bytes_for_G1_point;

pub struct Signature {
    pub point: GroupG1,
}

impl Clone for Signature {
    fn clone(&self) -> Signature {
        let mut temp_s = GroupG1::new();
        temp_s.copy(&self.point);
        Signature {
            point: temp_s
        }
    }
}

impl Signature {
    // Signature = H_0(msg) * sk
    pub fn new(msg: &[u8], sk: &SecretKey) -> Self {
        let hash_point = hash_on_GroupG1(msg);
        let sig = hash_point.mul(&sk.x);
        Signature { point: sig }
    }

    pub fn verify(&self, msg: &[u8], pk: &PublicKey) -> bool {
        // TODO: Check if point exists on curve, maybe use `ECP::new_big`
        // and x cord of verkey
        if self.point.is_infinity() {
            return false;
        }
        let msg_hash_point = hash_on_GroupG1(msg);
        let mut lhs = ate_pairing(&GeneratorG2, &self.point);
        let mut rhs = ate_pairing(&pk.point, &msg_hash_point);
        lhs.equals(&mut rhs)
    }

    pub fn from_bytes(sig_bytes: &[u8]) -> Result<Signature, SerzDeserzError> {
        Ok(Signature {
            point: get_G1_point_from_bytes(sig_bytes)?
        })
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        get_bytes_for_G1_point(&self.point)
    }
}

#[cfg(test)]
mod tests {
    // TODO: Add tests for failure
    // TODO: Add more test vectors
    use super::*;
    use super::super::keys::Keypair;

    #[test]
    fn basic_sign_verify() {
        let keypair = Keypair::random();
        let sk = keypair.sk;
        let vk = keypair.pk;

        let messages = vec![
            "",
            "a",
            "an example",
        ];

        for m in messages {
            /*
             * Simple sign and verify
             */
            let bytes = m.as_bytes();
            let mut sig = Signature::new(&bytes, &sk);
            assert!(sig.verify(&bytes, &vk));

            /*
             * Test serializing, then deserializing the signature
             */
            let sig_bytes = sig.to_bytes();
            let mut new_sig = Signature::from_bytes(&sig_bytes).unwrap();
            assert_eq!(&sig.point.tostring(), &new_sig.point.tostring());
            assert!(new_sig.verify(&bytes, &vk));
        }
    }

    #[test]
    fn verification_failure() {
        let keypair = Keypair::random();
        let sk = keypair.sk;
        let vk = keypair.pk;

        let mut msg = "Some msg";
        let sig = Signature::new(&msg.as_bytes(), &sk);
        msg = "Other msg";
        assert_eq!(sig.verify(&msg.as_bytes(), &vk), false);
        msg = "";
        assert_eq!(sig.verify(&msg.as_bytes(), &vk), false);
    }

    #[test]
    fn signature_at_infinity() {
        let keypair = Keypair::random();
        let vk = keypair.pk;

        let msg = "Small msg".as_bytes();
        let mut sig = Signature { point: GroupG1::new() };
        sig.point.inf();
        assert_eq!(sig.verify(&msg, &vk), false);
    }
}