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

use rgbstd::persistence::Stock;
use strict_encoding::{self, StrictDecode, StrictEncode};
use strict_encoding2::{
    DecodeError, StrictDecode as _, StrictEncode as _, StrictReader, StrictWriter,
};

#[derive(Clone, Debug, Default)]
pub enum RgbProxy {
    #[default]
    None,
    RgbV0_10(Stock),
}

impl RgbProxy {
    pub fn none() -> RgbProxy { RgbProxy::None }
    pub fn new() -> RgbProxy { RgbProxy::RgbV0_10(Stock::default()) }
    pub fn with(support_rgb: bool) -> RgbProxy {
        match support_rgb {
            true => Self::new(),
            false => Self::none(),
        }
    }
}

impl StrictEncode for RgbProxy {
    fn strict_encode<E: Write>(&self, mut e: E) -> Result<usize, strict_encoding::Error> {
        match self {
            RgbProxy::None => {
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
            0x0000 => Ok(RgbProxy::None),
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
