use std::{
    cmp::Ordering,
    collections::{BinaryHeap, HashMap, VecDeque},
};

use reth_primitives::{Address, PooledTransactionsElement};

use crate::AnyError;

type Nonce = u64;
type Tx = PooledTransactionsElement;

#[derive(Debug, Default)]
pub struct Nonces(HashMap<Address, Nonce>);

#[derive(Debug, Default)]
pub struct TxPool {
    scheduled: VecDeque<Tx>,
    pending: HashMap<Address, BinaryHeap<PendingTx>>,
}

impl TxPool {
    pub fn add(&mut self, nonces: &mut Nonces, tx: Tx) -> Result<(), AnyError> {
        let from_address = tx.recover_signer().ok_or("couldn't recover signer")?;
        let expected_nonce = nonces
            .get_mut(&from_address)
            .ok_or(format!("nonce unknown for the address: {}", from_address))?;

        match tx.nonce().cmp(expected_nonce) {
            Ordering::Equal => {
                use std::collections::hash_map::Entry as HMEntry;

                self.scheduled.push_back(tx);
                *expected_nonce += 1;

                if let HMEntry::Occupied(mut occupied) = self.pending.entry(from_address) {
                    let heap = occupied.get_mut();
                    while heap
                        .peek()
                        .map(|tx| tx.as_ref().nonce() == *expected_nonce)
                        .unwrap_or_default()
                    {
                        let tx = heap
                            .pop()
                            .expect("`<bool as Default>::default()` is `false`?");

                        self.scheduled.push_back(tx.into());
                        *expected_nonce += 1;
                    }
                    if heap.is_empty() {
                        occupied.remove();
                    }
                }

                Ok(())
            }
            Ordering::Greater => {
                let heap = self
                    .pending
                    .entry(from_address)
                    .or_insert(Default::default());
                heap.push(tx.into());
                Ok(())
            }
            Ordering::Less => Err("duplicate nonce".into()),
        }
    }

    pub fn scheduled_drain(&mut self) -> impl Iterator<Item = Tx> + '_ {
        std::iter::from_fn(|| self.scheduled.pop_front())
    }

    pub fn scheduled_count(&self) -> usize {
        self.scheduled.len()
    }
    pub fn pending_count(&self) -> usize {
        self.pending.values().map(BinaryHeap::len).sum()
    }
}

impl Nonces {
    pub fn get(&self, address: &Address) -> Option<u64> {
        self.0.get(address).copied()
    }
    pub fn get_mut(&mut self, address: &Address) -> Option<&mut u64> {
        self.0.get_mut(address)
    }

    pub fn ensure_for_address(
        &mut self,
        address: Address,
        fetch_fn: impl FnOnce(Address) -> Result<u64, AnyError>,
    ) -> Result<(), AnyError> {
        use std::collections::hash_map::Entry;
        let Entry::Vacant(entry) = self.0.entry(address) else {
            return Ok(());
        };
        let nonce = fetch_fn(address)?;
        entry.insert(nonce);
        Ok(())
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
impl AsRef<Tx> for PendingTx {
    fn as_ref(&self) -> &Tx {
        &self.0
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
