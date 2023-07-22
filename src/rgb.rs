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

use std::collections::BTreeSet;
use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};

use rgbstd::persistence::Stock;
use strict_encoding::{self, StrictDecode, StrictEncode};
use strict_encoding2::{
    DecodeError, StrictDecode as _, StrictEncode as _, StrictReader, StrictWriter,
};

use crate::OnchainTxid;

#[derive(Clone, Debug)]
pub enum RgbProxy {
    None {
        stock: Stock,
        witness_txes: BTreeSet<OnchainTxid>,
    },
    RgbV0_10 {
        stock: Stock,
        witness_txes: BTreeSet<OnchainTxid>,
    },
}

impl Deref for RgbProxy {
    type Target = Stock;

    fn deref(&self) -> &Self::Target {
        match self {
            RgbProxy::None { stock, .. } => stock,
            RgbProxy::RgbV0_10 { stock, .. } => stock,
        }
    }
}

impl DerefMut for RgbProxy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            RgbProxy::None { .. } => panic!("writing RGB stock in non-RGB wallet"),
            RgbProxy::RgbV0_10 { stock, .. } => stock,
        }
    }
}

impl Default for RgbProxy {
    fn default() -> Self { RgbProxy::none() }
}

impl RgbProxy {
    pub fn none() -> RgbProxy {
        RgbProxy::None {
            stock: none!(),
            witness_txes: none!(),
        }
    }
    pub fn new() -> RgbProxy {
        RgbProxy::RgbV0_10 {
            stock: empty!(),
            witness_txes: empty!(),
        }
    }
    pub fn with(support_rgb: bool) -> RgbProxy {
        match support_rgb {
            true => Self::new(),
            false => Self::none(),
        }
    }
    pub fn is_rgb(&self) -> bool {
        match self {
            RgbProxy::None { .. } => false,
            RgbProxy::RgbV0_10 { .. } => true,
        }
    }
    pub fn witness_txes(&self) -> &BTreeSet<OnchainTxid> {
        match self {
            RgbProxy::None { witness_txes, .. } => witness_txes,
            RgbProxy::RgbV0_10 { witness_txes, .. } => witness_txes,
        }
    }
    pub fn witness_txes_mut(&mut self) -> &mut BTreeSet<OnchainTxid> {
        match self {
            RgbProxy::None { .. } => panic!("writing RGB witntess index in non-RGB wallet"),
            RgbProxy::RgbV0_10 { witness_txes, .. } => witness_txes,
        }
    }
}

impl StrictEncode for RgbProxy {
    fn strict_encode<E: Write>(&self, mut e: E) -> Result<usize, strict_encoding::Error> {
        match self {
            RgbProxy::None { .. } => {
                e.write_all(&[0, 0])?;
                Ok(2)
            }
            RgbProxy::RgbV0_10 {
                stock,
                witness_txes,
            } => {
                e.write_all(&[1, 0])?;
                let counter = StrictWriter::with(u32::MAX as usize, e);
                let counter = stock.strict_encode(counter)?;
                let mut count = counter.count();
                let writer = counter.unbox();
                count += StrictEncode::strict_encode(witness_txes, writer)?;
                Ok(count)
            }
        }
    }
}

impl StrictDecode for RgbProxy {
    fn strict_decode<D: Read>(mut d: D) -> Result<Self, strict_encoding::Error> {
        match <u16 as StrictDecode>::strict_decode(&mut d)? {
            0x0000 => Ok(RgbProxy::none()),
            0x0001 => {
                let mut counter = StrictReader::with(u32::MAX as usize, d);
                let stock = Stock::strict_decode(&mut counter).map_err(|err| match err {
                    DecodeError::Io(io) => strict_encoding::Error::Io(io.kind().into()),
                    DecodeError::DataIntegrityError(e) => {
                        strict_encoding::Error::DataIntegrityError(e)
                    }
                    other => strict_encoding::Error::DataIntegrityError(other.to_string()),
                })?;
                let reader = counter.unbox();
                let witness_txes = StrictDecode::strict_decode(reader)?;
                Ok(RgbProxy::RgbV0_10 {
                    stock,
                    witness_txes,
                })
            }
            wrong => Err(strict_encoding::Error::DataIntegrityError(format!(
                "unsupported future version of wallet file (v{wrong})"
            ))),
        }
    }
}
