// This file is part of Substrate.

// Copyright (C) 2019-2021 Parity Technologies (UK) Ltd.
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

//! Tests for the im-online module.

#![cfg(test)]

use super::*;
use crate::mock::*;
use frame_support::{assert_noop, dispatch};
use sp_core::offchain::{
    testing::{TestOffchainExt, TestTransactionPoolExt},
    OffchainExt, TransactionPoolExt,
};
use sp_core::OpaquePeerId;
use sp_runtime::{testing::UintAuthorityId, transaction_validity::TransactionValidityError};

#[test]
fn test_unresponsiveness_slash_fraction() {
    // A single case of unresponsiveness is not slashed.
    assert_eq!(
        UnresponsivenessOffence::<()>::slash_fraction(1, 50),
        Perbill::zero(),
    );

    assert_eq!(
        UnresponsivenessOffence::<()>::slash_fraction(5, 50),
        Perbill::zero(), // 0%
    );

    assert_eq!(
        UnresponsivenessOffence::<()>::slash_fraction(7, 50),
        Perbill::from_parts(4200000), // 0.42%
    );

    // One third offline should be punished around 5%.
    assert_eq!(
        UnresponsivenessOffence::<()>::slash_fraction(17, 50),
        Perbill::from_parts(46200000), // 4.62%
    );
}

#[test]
fn should_report_offline_validators() {
    new_test_ext().execute_with(|| {
        // given
        let block = 1;
        System::set_block_number(block);
        // buffer new validators
        advance_session();
        // enact the change and buffer another one
        let validators = vec![1, 2, 3, 4, 5, 6];
        VALIDATORS.with(|l| *l.borrow_mut() = Some(validators.clone()));
        advance_session();

        // when
        // we end current session and start the next one
        advance_session();

        // then
        let offences = OFFENCES.with(|l| l.replace(vec![]));
        assert_eq!(
            offences,
            vec![(
                vec![],
                UnresponsivenessOffence {
                    session_index: 2,
                    validator_set_count: 3,
                    offenders: vec![(1, 1), (2, 2), (3, 3),],
                }
            )]
        );

        // should not report when heartbeat is sent
        for (idx, v) in validators.into_iter().take(4).enumerate() {
            let _ = heartbeat(block, 3, idx as u32, v.into(), Session::validators()).unwrap();
        }
        advance_session();

        // then
        let offences = OFFENCES.with(|l| l.replace(vec![]));
        assert_eq!(
            offences,
            vec![(
                vec![],
                UnresponsivenessOffence {
                    session_index: 3,
                    validator_set_count: 6,
                    offenders: vec![(5, 5), (6, 6),],
                }
            )]
        );
    });
}

fn heartbeat(
    block_number: u64,
    session_index: u32,
    authority_index: u32,
    id: UintAuthorityId,
    validators: Vec<u64>,
) -> dispatch::DispatchResult {
    use frame_support::unsigned::ValidateUnsigned;

    let heartbeat = Heartbeat {
        block_number,
        network_state: OpaqueNetworkState {
            peer_id: OpaquePeerId(vec![1]),
            external_addresses: vec![],
        },
        session_index,
        authority_index,
        validators_len: validators.len() as u32,
    };
    let signature = id.sign(&heartbeat.encode()).unwrap();

    ImOnline::pre_dispatch(&crate::Call::heartbeat(
        heartbeat.clone(),
        signature.clone(),
    ))
    .map_err(|e| match e {
        TransactionValidityError::Invalid(InvalidTransaction::Custom(INVALID_VALIDATORS_LEN)) => {
            "invalid validators len"
        }
        e @ _ => <&'static str>::from(e),
    })?;
    ImOnline::heartbeat(Origin::none(), heartbeat, signature)
}

#[test]
fn should_mark_online_validator_when_heartbeat_is_received() {
    new_test_ext().execute_with(|| {
        advance_session();
        // given
        VALIDATORS.with(|l| *l.borrow_mut() = Some(vec![1, 2, 3, 4, 5, 6]));
        assert_eq!(Session::validators(), Vec::<u64>::new());
        // enact the change and buffer another one
        advance_session();

        assert_eq!(Session::current_index(), 2);
        assert_eq!(Session::validators(), vec![1, 2, 3]);

        assert!(!ImOnline::is_online(0));
        assert!(!ImOnline::is_online(1));
        assert!(!ImOnline::is_online(2));

        // when
        let _ = heartbeat(1, 2, 0, 1.into(), Session::validators()).unwrap();

        // then
        assert!(ImOnline::is_online(0));
        assert!(!ImOnline::is_online(1));
        assert!(!ImOnline::is_online(2));

        // and when
        let _ = heartbeat(1, 2, 2, 3.into(), Session::validators()).unwrap();

        // then
        assert!(ImOnline::is_online(0));
        assert!(!ImOnline::is_online(1));
        assert!(ImOnline::is_online(2));
    });
}

#[test]
fn late_heartbeat_and_invalid_keys_len_should_fail() {
    new_test_ext().execute_with(|| {
        advance_session();
        // given
        VALIDATORS.with(|l| *l.borrow_mut() = Some(vec![1, 2, 3, 4, 5, 6]));
        assert_eq!(Session::validators(), Vec::<u64>::new());
        // enact the change and buffer another one
        advance_session();

        assert_eq!(Session::current_index(), 2);
        assert_eq!(Session::validators(), vec![1, 2, 3]);

        // when
        assert_noop!(
            heartbeat(1, 3, 0, 1.into(), Session::validators()),
            "Transaction is outdated"
        );
        assert_noop!(
            heartbeat(1, 1, 0, 1.into(), Session::validators()),
            "Transaction is outdated"
        );

        // invalid validators_len
        assert_noop!(
            heartbeat(1, 2, 0, 1.into(), vec![]),
            "invalid validators len"
        );
    });
}

#[test]
fn should_generate_heartbeats() {
    use frame_support::traits::OffchainWorker;

    let mut ext = new_test_ext();
    let (offchain, _state) = TestOffchainExt::new();
    let (pool, state) = TestTransactionPoolExt::new();
    ext.register_extension(OffchainExt::new(offchain));
    ext.register_extension(TransactionPoolExt::new(pool));

    ext.execute_with(|| {
        // given
        let block = 1;
        System::set_block_number(block);
        UintAuthorityId::set_all_keys(vec![0, 1, 2]);
        // buffer new validators
        Session::rotate_session();
        // enact the change and buffer another one
        VALIDATORS.with(|l| *l.borrow_mut() = Some(vec![1, 2, 3, 4, 5, 6]));
        Session::rotate_session();

        // when
        ImOnline::offchain_worker(block);

        // then
        let transaction = state.write().transactions.pop().unwrap();
        // All validators have `0` as their session key, so we generate 2 transactions.
        assert_eq!(state.read().transactions.len(), 2);

        // check stuff about the transaction.
        let ex: Extrinsic = Decode::decode(&mut &*transaction).unwrap();
        let heartbeat = match ex.call {
            crate::mock::Call::ImOnline(crate::Call::heartbeat(h, ..)) => h,
            e => panic!("Unexpected call: {:?}", e),
        };

        assert_eq!(
            heartbeat,
            Heartbeat {
                block_number: block,
                network_state: sp_io::offchain::network_state().unwrap(),
                session_index: 2,
                authority_index: 2,
                validators_len: 3,
            }
        );
    });
}

#[test]
fn should_cleanup_received_heartbeats_on_session_end() {
    new_test_ext().execute_with(|| {
        advance_session();

        VALIDATORS.with(|l| *l.borrow_mut() = Some(vec![1, 2, 3]));
        assert_eq!(Session::validators(), Vec::<u64>::new());

        // enact the change and buffer another one
        advance_session();

        assert_eq!(Session::current_index(), 2);
        assert_eq!(Session::validators(), vec![1, 2, 3]);

        // send an heartbeat from authority id 0 at session 2
        let _ = heartbeat(1, 2, 0, 1.into(), Session::validators()).unwrap();

        // the heartbeat is stored
        assert!(!ImOnline::received_heartbeats(&2, &0).is_none());

        advance_session();

        // after the session has ended we have already processed the heartbeat
        // message, so any messages received on the previous session should have
        // been pruned.
        assert!(ImOnline::received_heartbeats(&2, &0).is_none());
    });
}

#[test]
fn should_mark_online_validator_when_block_is_authored() {
    use pallet_authorship::EventHandler;

    new_test_ext().execute_with(|| {
        advance_session();
        // given
        VALIDATORS.with(|l| *l.borrow_mut() = Some(vec![1, 2, 3, 4, 5, 6]));
        assert_eq!(Session::validators(), Vec::<u64>::new());
        // enact the change and buffer another one
        advance_session();

        assert_eq!(Session::current_index(), 2);
        assert_eq!(Session::validators(), vec![1, 2, 3]);

        for i in 0..3 {
            assert!(!ImOnline::is_online(i));
        }

        // when
        ImOnline::note_author(1);
        ImOnline::note_uncle(2, 0);

        // then
        assert!(ImOnline::is_online(0));
        assert!(ImOnline::is_online(1));
        assert!(!ImOnline::is_online(2));
    });
}

#[test]
fn should_not_send_a_report_if_already_online() {
    use pallet_authorship::EventHandler;

    let mut ext = new_test_ext();
    let (offchain, _state) = TestOffchainExt::new();
    let (pool, pool_state) = TestTransactionPoolExt::new();
    ext.register_extension(OffchainExt::new(offchain));
    ext.register_extension(TransactionPoolExt::new(pool));

    ext.execute_with(|| {
        advance_session();
        // given
        VALIDATORS.with(|l| *l.borrow_mut() = Some(vec![1, 2, 3, 4, 5, 6]));
        assert_eq!(Session::validators(), Vec::<u64>::new());
        // enact the change and buffer another one
        advance_session();
        assert_eq!(Session::current_index(), 2);
        assert_eq!(Session::validators(), vec![1, 2, 3]);
        ImOnline::note_author(2);
        ImOnline::note_uncle(3, 0);

        // when
        UintAuthorityId::set_all_keys(vec![1, 2, 3]);
        // we expect error, since the authority is already online.
        let mut res = ImOnline::send_heartbeats(4).unwrap();
        assert_eq!(res.next().unwrap().unwrap(), ());
        assert_eq!(
            res.next().unwrap().unwrap_err(),
            OffchainErr::AlreadyOnline(1)
        );
        assert_eq!(
            res.next().unwrap().unwrap_err(),
            OffchainErr::AlreadyOnline(2)
        );
        assert_eq!(res.next(), None);

        // then
        let transaction = pool_state.write().transactions.pop().unwrap();
        // All validators have `0` as their session key, but we should only produce 1 heartbeat.
        assert_eq!(pool_state.read().transactions.len(), 0);
        // check stuff about the transaction.
        let ex: Extrinsic = Decode::decode(&mut &*transaction).unwrap();
        let heartbeat = match ex.call {
            crate::mock::Call::ImOnline(crate::Call::heartbeat(h, ..)) => h,
            e => panic!("Unexpected call: {:?}", e),
        };

        assert_eq!(
            heartbeat,
            Heartbeat {
                block_number: 4,
                network_state: sp_io::offchain::network_state().unwrap(),
                session_index: 2,
                authority_index: 0,
                validators_len: 3,
            }
        );
    });
}
