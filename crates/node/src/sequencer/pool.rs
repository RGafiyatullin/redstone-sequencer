use std::collections::{BTreeMap, BinaryHeap, HashMap, VecDeque};

use reth_primitives::Address;

use crate::AnyError;

type Tx = reth_primitives::PooledTransactionsElement;
type Nonce = u64;

#[derive(Debug, Default)]
pub struct Nonces(HashMap<Address, Nonce>);

#[derive(Debug, Default)]
pub struct Pool {
    scheduled: VecDeque<Tx>,
    pending: HashMap<Address, BinaryHeap<PendingTx>>,
}

impl Pool {
    pub fn add(&mut self, nonces: &mut Nonces, tx: Tx) -> Result<(), AnyError> {
        let from_address = tx.recover_signer().ok_or("couldn't recover signer")?;
        let expected_nonce = nonces
            .get_mut(&from_address)
            .ok_or(format!("nonce unknown for the address: {}", from_address))?;

        if *expected_nonce == tx.nonce() {
            *expected_nonce += 1;
            self.scheduled.push_back(tx);

            Ok(())
        } else {
            let heap = self
                .pending
                .entry(from_address)
                .or_insert(Default::default());
            heap.push(tx.into());
            Ok(())
        }
    }
}

impl Nonces {
    pub fn get_mut(&mut self, address: &Address) -> Option<&mut u64> {
        self.0.get_mut(address)
    }
}

#[derive(Debug)]
struct PendingTx(Tx);
impl From<Tx> for PendingTx {
    fn from(value: Tx) -> Self {
        Self(value)
    }
}
impl From<PendingTx> for Tx {
    fn from(value: PendingTx) -> Self {
        value.0
    }
}

impl std::cmp::Eq for PendingTx {}
impl std::cmp::Ord for PendingTx {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Reverse;
        Reverse(self.0.nonce()).cmp(&Reverse(other.0.nonce()))
    }
}

impl std::cmp::PartialEq for PendingTx {
    fn eq(&self, other: &Self) -> bool {
        self.0.nonce().eq(&other.0.nonce())
    }
}

impl std::cmp::PartialOrd for PendingTx {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
