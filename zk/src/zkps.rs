use anyhow::{anyhow, Result};
use bulletproofs::{BulletproofGens, PedersenGens, RangeProof};
use curve25519_dalek_ng::ristretto::CompressedRistretto;
use curve25519_dalek_ng::scalar::Scalar;
use merlin::Transcript;
use rand::rngs::OsRng;

pub struct RangeProver {
    bp_gens: BulletproofGens,
    pc_gens: PedersenGens,
}

impl Default for RangeProver {
    fn default() -> Self {
        Self {
            bp_gens: BulletproofGens::new(64, 1),
            pc_gens: PedersenGens::default(),
        }
    }
}

pub struct RangeProofData {
    pub proof: Vec<u8>,
    pub commitment: [u8; 32],
    pub bits: usize,
}

impl RangeProver {
    pub fn prove_amount(&self, amount: u64, bits: usize) -> Result<RangeProofData> {
        if bits > 64 || bits == 0 { return Err(anyhow!("bits must be 1..=64")); }
        let mut rng = OsRng;
        let blinding = Scalar::random(&mut rng);
        let mut transcript = Transcript::new(b"zkchain-rangeproof");
        let (proof, commit) = RangeProof::prove_single(
            &self.bp_gens,
            &self.pc_gens,
            &mut transcript,
            amount,
            &blinding,
            bits,
        )?;
        Ok(RangeProofData {
            proof: proof.to_bytes(),
            commitment: commit.to_bytes(),
            bits,
        })
    }

    pub fn verify_amount(&self, proof: &[u8], commitment: [u8; 32], bits: usize) -> Result<()> {
        if bits > 64 || bits == 0 { return Err(anyhow!("bits must be 1..=64")); }
        let proof = RangeProof::from_bytes(proof)?;
        let mut transcript = Transcript::new(b"zkchain-rangeproof");
        let commit = CompressedRistretto(commitment);
        proof.verify_single(&self.bp_gens, &self.pc_gens, &mut transcript, &commit, bits)?;
        Ok(())
    }
}
