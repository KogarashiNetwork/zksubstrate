// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
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

//! Timestamp pallet benchmarking.

#![cfg(feature = "runtime-benchmarks")]

use super::*;
use frame_benchmarking::{benchmarks, TrackedStorageKey};
use frame_support::{ensure, traits::OnFinalize};
use frame_system::RawOrigin;
use sp_std::prelude::*;

use crate::Module as Timestamp;

const MAX_TIME: u32 = 100;

benchmarks! {
    set {
        let t = MAX_TIME;
        // Ignore write to `DidUpdate` since it transient.
        let did_update_key = crate::DidUpdate::<T>::hashed_key().to_vec();
        frame_benchmarking::benchmarking::add_to_whitelist(TrackedStorageKey {
            key: did_update_key,
            has_been_read: false,
            has_been_written: true,
        });
    }: _(RawOrigin::None, t.into())
    verify {
        ensure!(Timestamp::<T>::now() == t.into(), "Time was not set.");
    }

    on_finalize {
        let t = MAX_TIME;
        Timestamp::<T>::set(RawOrigin::None.into(), t.into())?;
        ensure!(DidUpdate::<T>::exists(), "Time was not set.");
        // Ignore read/write to `DidUpdate` since it is transient.
        let did_update_key = crate::DidUpdate::<T>::hashed_key().to_vec();
        frame_benchmarking::benchmarking::add_to_whitelist(did_update_key.into());
    }: { Timestamp::<T>::on_finalize(t.into()); }
    verify {
        ensure!(!DidUpdate::<T>::exists(), "Time was not removed.");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::{new_test_ext, Test};
    use frame_support::assert_ok;

    #[test]
    fn test_benchmarks() {
        new_test_ext().execute_with(|| {
            assert_ok!(test_benchmark_set::<Test>());
            assert_ok!(test_benchmark_on_finalize::<Test>());
        });
    }
}
