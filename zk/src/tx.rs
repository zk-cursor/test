use anyhow::{Result, anyhow};
use ed25519_dalek::{Signer, Verifier, Signature, SigningKey, VerifyingKey};
use rand::rngs::OsRng;
use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};

use crate::zkps::{RangeProver, RangeProofData};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Address(pub [u8; 32]);

impl Address {
    pub fn from_public_key(pk: &VerifyingKey) -> Self {
        let mut h = Sha256::new();
        h.update(pk.to_bytes());
        let bytes = h.finalize();
        let mut addr = [0u8; 32];
        addr.copy_from_slice(&bytes);
        Address(addr)
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Tx {
    pub from: Address,
    pub to: Address,
    pub amount_commitment: [u8; 32],
    pub range_proof: Vec<u8>,
    pub bits: usize,
    pub signature: Vec<u8>,
    pub public_key: [u8; 32],
}

impl Tx {
    pub fn new_signed(
        from_sk: &SigningKey,
        to: Address,
        amount: u64,
        bits: usize,
        prover: &RangeProver,
    ) -> Result<Self> {
        let from = Address::from_public_key(&from_sk.verifying_key());
        let proof_data = prover.prove_amount(amount, bits)?;

        // sign message = hash(from||to||commitment||proof)
        let mut h = Sha256::new();
        h.update(&from.0);
        h.update(&to.0);
        h.update(&proof_data.commitment);
        h.update(&proof_data.proof);
        let msg = h.finalize();
        let sig = from_sk.sign(&msg);

        Ok(Tx {
            from,
            to,
            amount_commitment: proof_data.commitment,
            range_proof: proof_data.proof,
            bits: proof_data.bits,
            signature: sig.to_bytes().to_vec(),
            public_key: from_sk.verifying_key().to_bytes(),
        })
    }

    pub fn verify(&self, prover: &RangeProver) -> Result<()> {
        // verify range proof
        prover.verify_amount(&self.range_proof, self.amount_commitment, self.bits)?;
        // verify signature
        let vk = VerifyingKey::from_bytes(&self.public_key)?;
        let mut h = Sha256::new();
        h.update(&self.from.0);
        h.update(&self.to.0);
        h.update(&self.amount_commitment);
        h.update(&self.range_proof);
        let msg = h.finalize();
        let sig = { let arr: [u8;64] = self.signature.as_slice().try_into().map_err(|_| anyhow!("bad sig len"))?; Signature::from_bytes(&arr) };
        vk.verify(&msg, &sig)?;
        Ok(())
    }
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Account {
    pub address: Address,
    pub balance: u128,
}

pub fn generate_keypair() -> (SigningKey, VerifyingKey, Address) {
    let mut rng = OsRng;
    let sk = SigningKey::generate(&mut rng);
    let vk = sk.verifying_key();
    let addr = Address::from_public_key(&vk);
    (sk, vk, addr)
}
