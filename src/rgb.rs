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

use std::io::{Read, Write};
use std::ops::{Deref, DerefMut};

use rgbstd::persistence::Stock;
use strict_encoding::{self, StrictDecode, StrictEncode};
use strict_encoding2::{
    DecodeError, StrictDecode as _, StrictEncode as _, StrictReader, StrictWriter,
};

#[derive(Clone, Debug)]
pub enum RgbProxy {
    None(Stock),
    RgbV0_10(Stock),
}

impl Deref for RgbProxy {
    type Target = Stock;

    fn deref(&self) -> &Self::Target {
        match self {
            RgbProxy::None(stock) => stock,
            RgbProxy::RgbV0_10(stock) => stock,
        }
    }
}

impl DerefMut for RgbProxy {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            RgbProxy::None(_) => panic!("writing RGB stock in non-RGB wallet"),
            RgbProxy::RgbV0_10(stock) => stock,
        }
    }
}

impl Default for RgbProxy {
    fn default() -> Self { RgbProxy::none() }
}

impl RgbProxy {
    pub fn none() -> RgbProxy { RgbProxy::None(Stock::default()) }
    pub fn new() -> RgbProxy { RgbProxy::RgbV0_10(Stock::default()) }
    pub fn with(support_rgb: bool) -> RgbProxy {
        match support_rgb {
            true => Self::new(),
            false => Self::none(),
        }
    }
    pub fn is_rgb(&self) -> bool {
        match self {
            RgbProxy::None(_) => false,
            RgbProxy::RgbV0_10(_) => true,
        }
    }
}

impl StrictEncode for RgbProxy {
    fn strict_encode<E: Write>(&self, mut e: E) -> Result<usize, strict_encoding::Error> {
        match self {
            RgbProxy::None(_) => {
                e.write_all(&[0, 0])?;
                Ok(2)
            }
            RgbProxy::RgbV0_10(stock) => {
                e.write_all(&[1, 0])?;
                let counter = StrictWriter::with(u32::MAX as usize, e);
                let counter = stock.strict_encode(counter)?;
                Ok(counter.count())
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
                Ok(RgbProxy::RgbV0_10(stock))
            }
            wrong => Err(strict_encoding::Error::DataIntegrityError(format!(
                "unsupported future version of wallet file (v{wrong})"
            ))),
        }
    }
}
