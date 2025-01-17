// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
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

use libp2p::core::multiaddr::{Multiaddr, Protocol};
use std::collections::HashMap;

use sc_network::PeerId;
use sp_authority_discovery::AuthorityId;

/// Cache for [`AuthorityId`] -> [`Vec<Multiaddr>`] and [`PeerId`] -> [`AuthorityId`] mappings.
pub(super) struct AddrCache {
    authority_id_to_addresses: HashMap<AuthorityId, Vec<Multiaddr>>,
    peer_id_to_authority_id: HashMap<PeerId, AuthorityId>,
}

impl AddrCache {
    pub fn new() -> Self {
        AddrCache {
            authority_id_to_addresses: HashMap::new(),
            peer_id_to_authority_id: HashMap::new(),
        }
    }

    /// Inserts the given [`AuthorityId`] and [`Vec<Multiaddr>`] pair for future lookups by
    /// [`AuthorityId`] or [`PeerId`].
    pub fn insert(&mut self, authority_id: AuthorityId, mut addresses: Vec<Multiaddr>) {
        if addresses.is_empty() {
            return;
        }

        // Insert into `self.peer_id_to_authority_id`.
        let peer_ids = addresses
            .iter()
            .map(|a| peer_id_from_multiaddr(a))
            .filter_map(|peer_id| peer_id);
        for peer_id in peer_ids {
            self.peer_id_to_authority_id
                .insert(peer_id, authority_id.clone());
        }

        // Insert into `self.authority_id_to_addresses`.
        addresses.sort_unstable_by(|a, b| a.as_ref().cmp(b.as_ref()));
        self.authority_id_to_addresses
            .insert(authority_id, addresses);
    }

    /// Returns the number of authority IDs in the cache.
    pub fn num_ids(&self) -> usize {
        self.authority_id_to_addresses.len()
    }

    /// Returns the addresses for the given [`AuthorityId`].
    pub fn get_addresses_by_authority_id(
        &self,
        authority_id: &AuthorityId,
    ) -> Option<&Vec<Multiaddr>> {
        self.authority_id_to_addresses.get(&authority_id)
    }

    /// Returns the [`AuthorityId`] for the given [`PeerId`].
    pub fn get_authority_id_by_peer_id(&self, peer_id: &PeerId) -> Option<&AuthorityId> {
        self.peer_id_to_authority_id.get(peer_id)
    }

    /// Removes all [`PeerId`]s and [`Multiaddr`]s from the cache that are not related to the given
    /// [`AuthorityId`]s.
    pub fn retain_ids(&mut self, authority_ids: &Vec<AuthorityId>) {
        // The below logic could be replaced by `BtreeMap::drain_filter` once it stabilized.
        let authority_ids_to_remove = self
            .authority_id_to_addresses
            .iter()
            .filter(|(id, _addresses)| !authority_ids.contains(id))
            .map(|entry| entry.0)
            .cloned()
            .collect::<Vec<AuthorityId>>();

        for authority_id_to_remove in authority_ids_to_remove {
            // Remove other entries from `self.authority_id_to_addresses`.
            let addresses = self
                .authority_id_to_addresses
                .remove(&authority_id_to_remove);

            // Remove other entries from `self.peer_id_to_authority_id`.
            let peer_ids = addresses
                .iter()
                .flatten()
                .map(|a| peer_id_from_multiaddr(a))
                .filter_map(|peer_id| peer_id);
            for peer_id in peer_ids {
                if let Some(id) = self.peer_id_to_authority_id.remove(&peer_id) {
                    debug_assert_eq!(authority_id_to_remove, id);
                }
            }
        }
    }
}

fn peer_id_from_multiaddr(addr: &Multiaddr) -> Option<PeerId> {
    addr.iter().last().and_then(|protocol| {
        if let Protocol::P2p(multihash) = protocol {
            PeerId::from_multihash(multihash).ok()
        } else {
            None
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    use libp2p::multihash::{self, Multihash};
    use quickcheck::{Arbitrary, Gen, QuickCheck, TestResult};

    use sp_authority_discovery::{AuthorityId, AuthorityPair};
    use sp_core::crypto::Pair;

    #[derive(Clone, Debug)]
    struct TestAuthorityId(AuthorityId);

    impl Arbitrary for TestAuthorityId {
        fn arbitrary(g: &mut Gen) -> Self {
            let seed = (0..32).map(|_| u8::arbitrary(g)).collect::<Vec<_>>();
            TestAuthorityId(AuthorityPair::from_seed_slice(&seed).unwrap().public())
        }
    }

    #[derive(Clone, Debug)]
    struct TestMultiaddr(Multiaddr);

    impl Arbitrary for TestMultiaddr {
        fn arbitrary(g: &mut Gen) -> Self {
            let seed = (0..32).map(|_| u8::arbitrary(g)).collect::<Vec<_>>();
            let peer_id = PeerId::from_multihash(
                Multihash::wrap(multihash::Code::Sha2_256.into(), &seed).unwrap(),
            )
            .unwrap();
            let multiaddr = "/ip6/2001:db8:0:0:0:0:0:2/tcp/30333"
                .parse::<Multiaddr>()
                .unwrap()
                .with(Protocol::P2p(peer_id.into()));

            TestMultiaddr(multiaddr)
        }
    }

    #[test]
    fn retains_only_entries_of_provided_authority_ids() {
        fn property(
            first: (TestAuthorityId, TestMultiaddr),
            second: (TestAuthorityId, TestMultiaddr),
            third: (TestAuthorityId, TestMultiaddr),
        ) -> TestResult {
            let first: (AuthorityId, Multiaddr) = ((first.0).0, (first.1).0);
            let second: (AuthorityId, Multiaddr) = ((second.0).0, (second.1).0);
            let third: (AuthorityId, Multiaddr) = ((third.0).0, (third.1).0);

            let mut cache = AddrCache::new();

            cache.insert(first.0.clone(), vec![first.1.clone()]);
            cache.insert(second.0.clone(), vec![second.1.clone()]);
            cache.insert(third.0.clone(), vec![third.1.clone()]);

            assert_eq!(
                Some(&vec![third.1.clone()]),
                cache.get_addresses_by_authority_id(&third.0),
                "Expect `get_addresses_by_authority_id` to return addresses of third authority."
            );
            assert_eq!(
                Some(&third.0),
                cache.get_authority_id_by_peer_id(&peer_id_from_multiaddr(&third.1).unwrap()),
                "Expect `get_authority_id_by_peer_id` to return `AuthorityId` of third authority."
            );

            cache.retain_ids(&vec![first.0, second.0]);

            assert_eq!(
                None,
                cache.get_addresses_by_authority_id(&third.0),
                "Expect `get_addresses_by_authority_id` to not return `None` for third authority."
            );
            assert_eq!(
                None,
                cache.get_authority_id_by_peer_id(&peer_id_from_multiaddr(&third.1).unwrap()),
                "Expect `get_authority_id_by_peer_id` to return `None` for third authority."
            );

            TestResult::passed()
        }

        QuickCheck::new()
            .max_tests(10)
            .quickcheck(property as fn(_, _, _) -> TestResult)
    }
}
