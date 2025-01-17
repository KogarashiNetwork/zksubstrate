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

// Benchmarks for Indices Pallet

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use sp_runtime::traits::Bounded;

use crate::Module as Indices;

const SEED: u32 = 0;

benchmarks! {
    claim {
        let account_index = T::AccountIndex::from(SEED);
        let caller: T::AccountId = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
    }: _(RawOrigin::Signed(caller.clone()), account_index)
    verify {
        assert_eq!(Accounts::<T>::get(account_index).unwrap().0, caller);
    }

    transfer {
        let account_index = T::AccountIndex::from(SEED);
        // Setup accounts
        let caller: T::AccountId = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        let recipient: T::AccountId = account("recipient", 0, SEED);
        T::Currency::make_free_balance_be(&recipient, BalanceOf::<T>::max_value());
        // Claim the index
        Indices::<T>::claim(RawOrigin::Signed(caller.clone()).into(), account_index)?;
    }: _(RawOrigin::Signed(caller.clone()), recipient.clone(), account_index)
    verify {
        assert_eq!(Accounts::<T>::get(account_index).unwrap().0, recipient);
    }

    free {
        let account_index = T::AccountIndex::from(SEED);
        // Setup accounts
        let caller: T::AccountId = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        // Claim the index
        Indices::<T>::claim(RawOrigin::Signed(caller.clone()).into(), account_index)?;
    }: _(RawOrigin::Signed(caller.clone()), account_index)
    verify {
        assert_eq!(Accounts::<T>::get(account_index), None);
    }

    force_transfer {
        let account_index = T::AccountIndex::from(SEED);
        // Setup accounts
        let original: T::AccountId = account("original", 0, SEED);
        T::Currency::make_free_balance_be(&original, BalanceOf::<T>::max_value());
        let recipient: T::AccountId = account("recipient", 0, SEED);
        T::Currency::make_free_balance_be(&recipient, BalanceOf::<T>::max_value());
        // Claim the index
        Indices::<T>::claim(RawOrigin::Signed(original).into(), account_index)?;
    }: _(RawOrigin::Root, recipient.clone(), account_index, false)
    verify {
        assert_eq!(Accounts::<T>::get(account_index).unwrap().0, recipient);
    }

    freeze {
        let account_index = T::AccountIndex::from(SEED);
        // Setup accounts
        let caller: T::AccountId = whitelisted_caller();
        T::Currency::make_free_balance_be(&caller, BalanceOf::<T>::max_value());
        // Claim the index
        Indices::<T>::claim(RawOrigin::Signed(caller.clone()).into(), account_index)?;
    }: _(RawOrigin::Signed(caller.clone()), account_index)
    verify {
        assert_eq!(Accounts::<T>::get(account_index).unwrap().2, true);
    }

    // TODO in another PR: lookup and unlookup trait weights (not critical)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{new_test_ext, Test};
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        new_test_ext().execute_with(|| {
            assert_ok!(test_benchmark_claim::<Test>());
            assert_ok!(test_benchmark_transfer::<Test>());
            assert_ok!(test_benchmark_free::<Test>());
            assert_ok!(test_benchmark_force_transfer::<Test>());
            assert_ok!(test_benchmark_freeze::<Test>());
        });
    }
}
