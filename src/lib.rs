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

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate strict_encoding;
#[cfg(feature = "serde")]
extern crate serde_crate as serde;
#[cfg(feature = "serde")]
#[macro_use]
extern crate serde_with;

mod electrum;
pub mod file;
mod onchain;
pub mod psbt;
mod sign;
mod taptree;
mod template;
mod types;
mod wallet;

pub use electrum::{ElectrumPreset, ElectrumSec, ElectrumServer};
pub use file::FileDocument;
pub use onchain::{
    AddressSource, AddressSummary, AddressValue, HistoryEntry, OnchainStatus, OnchainTxid, Prevout,
    TxidMeta, UtxoTxid,
};
pub use sign::XprivSigner;
pub use taptree::ToTapTree;
pub use template::{Requirement, WalletTemplate};
pub use types::{
    Error, HardwareDevice, HardwareList, OriginFormat, Ownership, Signer, SigsReq,
    TimelockDuration, TimelockReq, TimelockedSigs,
};

pub use self::wallet::{
    DerivationStandardExt, DerivationType, DescriptorError, SpendingCondition, Wallet,
    WalletDescriptor, WalletEphemerals, WalletSettings, WalletState,
};
