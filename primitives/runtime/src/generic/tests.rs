// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Tests for the generic implementations of Extrinsic/Header/Block.

use super::DigestItem;
use crate::codec::{Decode, Encode};
use sp_core::H256;

#[test]
fn system_digest_item_encoding() {
    let item = DigestItem::ChangesTrieRoot::<H256>(H256::default());
    let encoded = item.encode();
    assert_eq!(
        encoded,
        vec![
            // type = DigestItemType::ChangesTrieRoot
            2, // trie root
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0,
        ]
    );

    let decoded: DigestItem<H256> = Decode::decode(&mut &encoded[..]).unwrap();
    assert_eq!(item, decoded);
}

#[test]
fn non_system_digest_item_encoding() {
    let item = DigestItem::Other::<H256>(vec![10, 20, 30]);
    let encoded = item.encode();
    assert_eq!(
        encoded,
        vec![
            // type = DigestItemType::Other
            0,  // length of other data
            12, // authorities
            10, 20, 30,
        ]
    );

    let decoded: DigestItem<H256> = Decode::decode(&mut &encoded[..]).unwrap();
    assert_eq!(item, decoded);
}
