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

use chrono::prelude::*;
use wallet::descriptors::DescriptorClass;
use wallet::hd::{Bip43, HardenedIndex, SegmentIndexes};
use wallet::onchain::PublicNetwork;

use crate::{DerivationType, SigsReq, SpendingCondition};

#[derive(Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Default)]
pub enum Requirement {
    #[default]
    Allow,
    Require,
    Deny,
}

/// Wallet template is a way to define constrained version of a wallet descriptor, but unlike
/// [`super::WalletDescriptor`] not having restrains on the internal consistency between amount of
/// signatures already present and condition parameters.
#[derive(Clone, Eq, PartialEq, Hash, Debug)]
pub struct WalletTemplate {
    pub default_derivation: DerivationType,
    pub descriptor_class: DescriptorClass,
    pub min_signer_count: u16,
    pub max_signer_count: Option<u16>,
    pub hardware_req: Requirement,
    pub watch_only_req: Requirement,
    pub conditions: BTreeSet<(u8, SpendingCondition)>,
    pub network: PublicNetwork,
    pub use_rgb: bool,
}

impl WalletTemplate {
    pub fn taproot_singlesig_rgb(network: PublicNetwork, require_hardware: bool) -> WalletTemplate {
        let hardware_req = match require_hardware {
            true => Requirement::Require,
            false => Requirement::Deny,
        };
        let watch_only_req = match require_hardware {
            true => Requirement::Deny,
            false => Requirement::Require,
        };
        WalletTemplate {
            default_derivation: Bip43::singlesig_taproot().into(),
            descriptor_class: DescriptorClass::TaprootC0,
            min_signer_count: 1,
            max_signer_count: Some(1),
            hardware_req,
            watch_only_req,
            conditions: bset![(0, SpendingCondition::default())],
            network,
            use_rgb: true,
        }
    }

    pub fn singlesig(
        descriptor_class: DescriptorClass,
        network: PublicNetwork,
        require_hardware: bool,
        use_rgb: bool,
    ) -> WalletTemplate {
        let format = descriptor_class.bip43(1);
        let hardware_req = match require_hardware {
            true => Requirement::Require,
            false => Requirement::Deny,
        };
        let watch_only_req = match require_hardware {
            true => Requirement::Deny,
            false => Requirement::Require,
        };
        WalletTemplate {
            default_derivation: format.into(),
            descriptor_class,
            min_signer_count: 1,
            max_signer_count: Some(1),
            hardware_req,
            watch_only_req,
            conditions: bset![(0, SpendingCondition::default())],
            network,
            use_rgb,
        }
    }

    /// # Panics
    ///
    /// If `sigs_required` is less than 3.
    pub fn hodling(
        descriptor_class: DescriptorClass,
        network: PublicNetwork,
        sigs_required: u16,
        hardware_req: Requirement,
        watch_only_req: Requirement,
    ) -> WalletTemplate {
        let now = Utc::now();
        if sigs_required < 3 {
            unreachable!("WalletTemplate::hodling must require at least 3 signers")
        }
        let conditions = bset![
            (1, SpendingCondition::all()),
            (
                2,
                SpendingCondition::anybody_after_date(now.with_year(now.year() + 5).unwrap())
            )
        ];
        WalletTemplate {
            default_derivation: Bip43::multisig_descriptor().into(),
            descriptor_class,
            min_signer_count: sigs_required,
            max_signer_count: None,
            hardware_req,
            watch_only_req,
            conditions,
            network,
            use_rgb: false,
        }
    }

    /// # Panics
    ///
    /// If `sigs_required` is `Some(0)` or `Some(1)`.
    pub fn multisig(
        descriptor_class: DescriptorClass,
        network: PublicNetwork,
        sigs_required: Option<u16>,
        hardware_req: Requirement,
        watch_only_req: Requirement,
    ) -> WalletTemplate {
        let now = Utc::now();
        let conditions = match sigs_required {
            None => bset![(0, SpendingCondition::default())],
            Some(0) | Some(1) => unreachable!("WalletTemplate::multisig must expect > 1 signature"),
            Some(2) => bset![
                (1, SpendingCondition::all()),
                (
                    2,
                    SpendingCondition::anybody_after_date(now.with_year(now.year() + 5).unwrap())
                )
            ],
            Some(3) => bset![
                (1, SpendingCondition::at_least(2)),
                (
                    2,
                    SpendingCondition::anybody_after_date(now.with_year(now.year() + 5).unwrap())
                )
            ],
            Some(count) => bset![
                (1, SpendingCondition::at_least(count - 1)),
                (
                    2,
                    SpendingCondition::after_date(
                        SigsReq::AtLeast(count / 2 + count % 2),
                        now.with_year(now.year() + 3).unwrap(),
                    )
                ),
                (
                    3,
                    SpendingCondition::anybody_after_date(now.with_year(now.year() + 5).unwrap())
                )
            ],
        };
        let default_derivation = match descriptor_class {
            DescriptorClass::PreSegwit => Bip43::multisig_ordered_sh(),
            DescriptorClass::SegwitV0 => Bip43::multisig_segwit0(),
            DescriptorClass::NestedV0 => Bip43::multisig_nested0(),
            DescriptorClass::TaprootC0 => Bip43::multisig_descriptor(),
        }
        .into();
        WalletTemplate {
            default_derivation,
            descriptor_class,
            min_signer_count: sigs_required.unwrap_or(2),
            max_signer_count: None,
            hardware_req,
            watch_only_req,
            conditions,
            network,
            use_rgb: false,
        }
    }

    pub fn bip43(&self) -> Bip43 {
        // TODO: Fix this
        self.default_derivation.bip43().unwrap_or(Bip43::Bip43 {
            purpose: HardenedIndex::zero(),
        })
    }
}
