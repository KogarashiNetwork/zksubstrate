// This file is part of Substrate.

// Copyright (C) 2018-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Aura (Authority-Round) digests
//!
//! This implements the digests for AuRa, to allow the private
//! `CompatibleDigestItem` trait to appear in public interfaces.

use codec::{Codec, Encode};
use sp_consensus_aura::AURA_ENGINE_ID;
use sp_consensus_slots::Slot;
use sp_core::Pair;
use sp_runtime::generic::{DigestItem, OpaqueDigestItemId};
use std::fmt::Debug;

type Signature<P> = <P as Pair>::Signature;

/// A digest item which is usable with aura consensus.
pub trait CompatibleDigestItem<P: Pair>: Sized {
    /// Construct a digest item which contains a signature on the hash.
    fn aura_seal(signature: Signature<P>) -> Self;

    /// If this item is an Aura seal, return the signature.
    fn as_aura_seal(&self) -> Option<Signature<P>>;

    /// Construct a digest item which contains the slot number
    fn aura_pre_digest(slot: Slot) -> Self;

    /// If this item is an AuRa pre-digest, return the slot number
    fn as_aura_pre_digest(&self) -> Option<Slot>;
}

impl<P, Hash> CompatibleDigestItem<P> for DigestItem<Hash>
where
    P: Pair,
    Signature<P>: Codec,
    Hash: Debug + Send + Sync + Eq + Clone + Codec + 'static,
{
    fn aura_seal(signature: Signature<P>) -> Self {
        DigestItem::Seal(AURA_ENGINE_ID, signature.encode())
    }

    fn as_aura_seal(&self) -> Option<Signature<P>> {
        self.try_to(OpaqueDigestItemId::Seal(&AURA_ENGINE_ID))
    }

    fn aura_pre_digest(slot: Slot) -> Self {
        DigestItem::PreRuntime(AURA_ENGINE_ID, slot.encode())
    }

    fn as_aura_pre_digest(&self) -> Option<Slot> {
        self.try_to(OpaqueDigestItemId::PreRuntime(&AURA_ENGINE_ID))
    }
}
