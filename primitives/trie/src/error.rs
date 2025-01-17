// This file is part of Substrate.

// Copyright (C) 2015-2021 Parity Technologies (UK) Ltd.
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

#[cfg(feature = "std")]
use std::error::Error as StdError;
#[cfg(feature = "std")]
use std::fmt;

#[derive(Debug, PartialEq, Eq, Clone)]
/// Error for trie node decoding.
pub enum Error {
    /// Bad format.
    BadFormat,
    /// Decoding error.
    Decode(codec::Error),
}

impl From<codec::Error> for Error {
    fn from(x: codec::Error) -> Self {
        Error::Decode(x)
    }
}

#[cfg(feature = "std")]
impl StdError for Error {
    fn description(&self) -> &str {
        match self {
            Error::BadFormat => "Bad format error",
            Error::Decode(_) => "Decoding error",
        }
    }
}

#[cfg(feature = "std")]
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Decode(e) => write!(f, "Decode error: {}", e),
            Error::BadFormat => write!(f, "Bad format"),
        }
    }
}
