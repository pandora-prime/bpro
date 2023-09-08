// Rust bitcoin wallet library for professional use.
//
// Written in 2022 by
//     Dr. Maxim Orlovsky <orlovsky@pandoraprime.ch>
//
// Copyright (C) 2022 by Pandora Prime SA, Switzerland.
//
// This software is distributed without any warranty. You should have received
// a copy of the AGPL-3.0 License along with this software. If not, see
// <https://www.gnu.org/licenses/agpl-3.0-standalone.html>.

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::hash::{Hash, Hasher};

use ::wallet::hd::{DerivationSubpath, SegmentIndexes, UnhardenedIndex};
use bitcoin::{OutPoint, Transaction, Txid};
use bitcoin_scripts::address::AddressCompat;
use bitcoin_scripts::PubkeyScript;
use chrono::{DateTime, NaiveDateTime, Utc};
#[cfg(feature = "electrum")]
use electrum_client::{GetHistoryRes, ListUnspentRes};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct AddressSummary {
    pub addr_src: AddressSource,
    pub balance: u64,
    pub volume: u64,
    pub tx_count: u32,
}

impl AddressSummary {
    pub fn merge(&mut self, other: AddressSummary) {
        self.balance += other.balance;
        self.volume += other.volume;
        self.tx_count += 1;
    }
}

impl AddressSummary {
    pub fn icon_name(self) -> Option<&'static str> { self.addr_src.icon_name() }

    pub fn terminal_string(self) -> String { self.addr_src.terminal_string() }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(StrictEncode, StrictDecode)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct AddressSource {
    #[cfg_attr(
        feature = "serde",
        serde(with = "::serde_with::As::<::serde_with::DisplayFromStr>")
    )]
    pub address: AddressCompat,
    pub change: UnhardenedIndex,
    pub index: UnhardenedIndex,
}

impl AddressSource {
    /// # Panics
    ///
    /// If the provided script can't be represented as an address.
    pub fn with(
        script: &PubkeyScript,
        index: UnhardenedIndex,
        change: bool,
        network: bitcoin::Network,
    ) -> AddressSource {
        AddressSource {
            address: AddressCompat::from_script(script, network.into()).expect("invalid script"),
            change: UnhardenedIndex::from(change as u8),
            index,
        }
    }

    pub fn icon_name(self) -> Option<&'static str> {
        match self.change.first_index() {
            1 => Some("view-refresh-symbolic"),
            _ => None,
        }
    }

    pub fn change_index(self) -> UnhardenedIndex { self.change }

    pub fn terminal_string(self) -> String { format!("/{}/{}", self.change_index(), self.index) }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(StrictEncode, StrictDecode)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct AddressValue {
    pub addr_src: AddressSource,
    pub value: u64,
}

impl AddressValue {
    pub fn icon_name(self) -> Option<&'static str> { self.addr_src.icon_name() }

    pub fn terminal_string(self) -> String { self.addr_src.terminal_string() }
}

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd, Ord, Hash, Debug)]
#[derive(StrictEncode, StrictDecode)]
#[cfg_attr(
    feature = "serde",
    derive(Serialize, Deserialize),
    serde(crate = "serde_crate", rename_all = "lowercase")
)]
pub enum OnchainStatus {
    Blockchain(u32),
    Mempool,
}

impl OnchainStatus {
    pub fn from_u32(index: u32) -> OnchainStatus {
        match index {
            0 => OnchainStatus::Mempool,
            height => OnchainStatus::Blockchain(height),
        }
    }

    pub fn from_i32(index: i32) -> OnchainStatus {
        match index {
            i if i <= 0 => OnchainStatus::Mempool,
            height => OnchainStatus::Blockchain(height as u32),
        }
    }

    pub fn into_u32(self) -> u32 {
        match self {
            OnchainStatus::Blockchain(height) => height,
            OnchainStatus::Mempool => 0,
        }
    }

    pub fn into_i32(self) -> i32 {
        match self {
            OnchainStatus::Blockchain(height) => height as i32,
            OnchainStatus::Mempool => 0,
        }
    }

    pub fn in_mempool(self) -> bool { self == OnchainStatus::Mempool }

    pub fn is_mined(self) -> bool { self != OnchainStatus::Mempool }

    // TODO: Do a binary file indexed by height, representing date/time information for each height
    pub fn date_time_est(self) -> DateTime<chrono::Local> {
        match self {
            OnchainStatus::Mempool => chrono::Local::now(),
            OnchainStatus::Blockchain(height) => {
                let reference_height = 733961;
                let reference_time = 1651158666_i32;
                let height_diff = height as i32 - reference_height;
                let timestamp = reference_time.saturating_add(height_diff * 600);
                let block_time = NaiveDateTime::from_timestamp_opt(timestamp as i64, 0)
                    .expect("invalid block timestamp");
                DateTime::<chrono::Local>::from(DateTime::<Utc>::from_utc(block_time, Utc))
            }
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Hash, Debug)]
#[derive(StrictEncode, StrictDecode)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct OnchainTxid {
    pub txid: Txid,
    pub status: OnchainStatus,
    pub date_time: Option<DateTime<Utc>>,
}

impl Ord for OnchainTxid {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.status.cmp(&other.status) {
            Ordering::Equal => self.txid.cmp(&other.txid),
            ordering => ordering,
        }
    }
}

impl PartialOrd for OnchainTxid {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.txid == other.txid && self.status != other.status {
            None
        } else {
            Some(self.cmp(other))
        }
    }
}

impl OnchainTxid {
    pub fn date_time_est(self) -> DateTime<chrono::Local> {
        self.date_time()
            .unwrap_or_else(|| self.status.date_time_est())
    }

    pub fn date_time(self) -> Option<DateTime<chrono::Local>> {
        self.date_time.map(DateTime::<chrono::Local>::from)
    }

    pub fn mining_info(self) -> String {
        match self.status {
            OnchainStatus::Mempool => s!("pending"),
            OnchainStatus::Blockchain(_) => format!("{}", self.date_time_est().format("%F %l %P")),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Hash, Debug)]
#[derive(StrictEncode, StrictDecode)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct Comment {
    pub label: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Clone, Eq, Debug)]
#[derive(StrictEncode, StrictDecode)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct HistoryEntry {
    /// For spending, txid of the transaction that spends wallet funds.
    /// For incoming payments (including change operations), txid containing funds on an address of
    /// the wallet.
    pub onchain: OnchainTxid,
    pub tx: Transaction,
    pub credit: BTreeMap<u32, AddressValue>,
    pub debit: BTreeMap<u32, AddressSource>,
    pub payers: BTreeMap<u32, (Option<String>, Option<AddressValue>)>,
    pub beneficiaries: BTreeMap<u32, String>,
    pub fee: Option<u64>,
    pub comment: Option<Comment>,
}

impl Hash for HistoryEntry {
    fn hash<H: Hasher>(&self, state: &mut H) { state.write(self.tx.txid().as_ref()) }
}

impl PartialEq for HistoryEntry {
    fn eq(&self, other: &Self) -> bool { self.tx.txid() == other.tx.txid() }
}

impl Ord for HistoryEntry {
    fn cmp(&self, other: &Self) -> Ordering { self.onchain.cmp(&other.onchain) }
}

impl PartialOrd for HistoryEntry {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.onchain.partial_cmp(&other.onchain)
    }
}

impl HistoryEntry {
    pub fn icon_name(&self) -> &'static str {
        match self.balance() {
            x if x > 0 => "media-playlist-consecutive-symbolic",
            x if x < 0 => "mail-send-symbolic",
            0 => "view-refresh-symbolic",
            _ => unreachable!(),
        }
    }

    pub fn date_time_est(&self) -> DateTime<chrono::Local> { self.onchain.date_time_est() }

    pub fn date_time(&self) -> Option<DateTime<chrono::Local>> { self.onchain.date_time() }

    pub fn mining_info(&self) -> String { self.onchain.mining_info() }

    pub fn value_credited(&self) -> u64 { self.credit.values().map(|addr| addr.value).sum() }

    pub fn value_debited(&self) -> u64 {
        self.debit
            .keys()
            .filter_map(|vout| self.tx.output.get(*vout as usize))
            .map(|txout| txout.value)
            .sum()
    }

    pub fn balance(&self) -> i64 { self.value_debited() as i64 - self.value_credited() as i64 }

    pub fn address_summaries(&self) -> Vec<AddressSummary> {
        self.credit
            .values()
            .map(|a| AddressSummary {
                addr_src: a.addr_src,
                balance: 0,
                volume: 0,
                tx_count: 1,
            })
            .chain(self.debit.iter().map(|(vout, a)| {
                AddressSummary {
                    addr_src: *a,
                    balance: 0,
                    volume: self
                        .tx
                        .output
                        .get(*vout as usize)
                        .map(|txout| txout.value)
                        .unwrap_or_default(),
                    tx_count: 1,
                }
            }))
            .collect()
    }

    pub fn set_comment(&mut self, label: String) {
        self.comment = Some(Comment {
            label,
            timestamp: Utc::now(),
        })
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
#[derive(StrictEncode, StrictDecode)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(crate = "serde_crate"))]
pub struct UtxoTxid {
    pub onchain: OnchainTxid,
    pub value: u64,
    pub vout: u32,
    pub addr_src: AddressSource,
}

impl UtxoTxid {
    pub fn outpoint(&self) -> OutPoint { OutPoint::new(self.onchain.txid, self.vout) }

    pub fn date_time_est(self) -> DateTime<chrono::Local> { self.onchain.date_time_est() }

    pub fn date_time(self) -> Option<DateTime<chrono::Local>> { self.onchain.date_time() }

    pub fn mining_info(self) -> String { self.onchain.mining_info() }
}

impl From<&UtxoTxid> for Prevout {
    fn from(utxo: &UtxoTxid) -> Prevout {
        Prevout {
            outpoint: utxo.outpoint(),
            amount: utxo.value,
            change: utxo.addr_src.change,
            index: utxo.addr_src.index,
        }
    }
}

impl From<UtxoTxid> for Prevout {
    fn from(utxo: UtxoTxid) -> Prevout { Prevout::from(&utxo) }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Prevout {
    pub outpoint: OutPoint,
    pub amount: u64,
    pub change: UnhardenedIndex,
    pub index: UnhardenedIndex,
}

impl Prevout {
    pub fn terminal(&self) -> DerivationSubpath<UnhardenedIndex> {
        DerivationSubpath::from(&[self.change, self.index][..])
    }
}

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug)]
pub struct TxidMeta {
    pub onchain: OnchainTxid,
    pub fee: Option<u64>,
}

impl From<&UtxoTxid> for TxidMeta {
    fn from(utxo: &UtxoTxid) -> Self {
        TxidMeta {
            onchain: utxo.onchain,
            fee: None,
        }
    }
}

#[cfg(feature = "electrum")]
impl From<GetHistoryRes> for TxidMeta {
    fn from(res: GetHistoryRes) -> Self {
        TxidMeta {
            onchain: OnchainTxid {
                txid: res.tx_hash,
                status: OnchainStatus::from_i32(res.height),
                date_time: None,
            },
            fee: res.fee,
        }
    }
}

#[cfg(feature = "electrum")]
impl From<&ListUnspentRes> for OnchainTxid {
    fn from(res: &ListUnspentRes) -> Self {
        OnchainTxid {
            txid: res.tx_hash,
            status: OnchainStatus::from_u32(res.height as u32),
            date_time: None,
        }
    }
}

#[cfg(feature = "electrum")]
impl UtxoTxid {
    pub fn with(res: ListUnspentRes, addr_src: AddressSource) -> Self {
        UtxoTxid {
            onchain: OnchainTxid::from(&res),
            vout: res.tx_pos as u32,
            value: res.value,
            addr_src,
        }
    }
}
