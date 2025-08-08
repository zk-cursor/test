use anyhow::{Result, anyhow};
use parking_lot::RwLock;
use serde::{Serialize, Deserialize};
use sha2::{Sha256, Digest};
use std::collections::HashMap;
use std::sync::Arc;
use crate::tx::{Tx, Address};
use crate::zkps::RangeProver;

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Block {
    pub height: u64,
    pub prev_hash: [u8; 32],
    pub txs: Vec<Tx>,
    pub hash: [u8; 32],
}

impl Block {
    pub fn compute_hash(height: u64, prev_hash: [u8; 32], txs: &Vec<Tx>) -> [u8; 32] {
        let mut h = Sha256::new();
        h.update(height.to_le_bytes());
        h.update(prev_hash);
        let bytes = serde_json::to_vec(&txs).unwrap();
        h.update(bytes);
        let out = h.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&out);
        hash
    }
}

#[derive(Default)]
pub struct LedgerState {
    pub balances: HashMap<[u8;32], u128>,
}

pub struct NodeState {
    pub chain: Vec<Block>,
    pub state: LedgerState,
    pub prover: RangeProver,
}

impl NodeState {
    pub fn new() -> Self {
        let genesis = Block { height: 0, prev_hash: [0u8;32], txs: vec![], hash: [0u8;32] };
        Self {
            chain: vec![genesis],
            state: LedgerState { balances: HashMap::new() },
            prover: RangeProver::default(),
        }
    }

    pub fn verify_tx(&self, tx: &Tx) -> Result<()> {
        tx.verify(&self.prover)
    }

    pub fn apply_block(&mut self, txs: Vec<Tx>) -> Result<Block> {
        for tx in &txs { self.verify_tx(tx)?; }
        let height = self.chain.len() as u64;
        let prev_hash = self.chain.last().map(|b| b.hash).unwrap_or([0u8;32]);
        let hash = Block::compute_hash(height, prev_hash, &txs);
        let block = Block { height, prev_hash, hash, txs };
        self.chain.push(block.clone());
        Ok(block)
    }
}

pub type SharedNode = Arc<RwLock<NodeState>>;

pub fn new_shared_node() -> SharedNode { Arc::new(RwLock::new(NodeState::new())) }
