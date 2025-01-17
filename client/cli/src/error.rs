// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
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

//! Initialization errors.

use sp_core::crypto;

/// Result type alias for the CLI.
pub type Result<T> = std::result::Result<T, Error>;

/// Error type for the CLI.
#[derive(Debug, thiserror::Error)]
#[allow(missing_docs)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Cli(#[from] structopt::clap::Error),

    #[error(transparent)]
    Service(#[from] sc_service::Error),

    #[error(transparent)]
    Client(#[from] sp_blockchain::Error),

    #[error(transparent)]
    Codec(#[from] parity_scale_codec::Error),

    #[error("Invalid input: {0}")]
    Input(String),

    #[error("Invalid listen multiaddress")]
    InvalidListenMultiaddress,

    #[error("Invalid URI; expecting either a secret URI or a public URI.")]
    InvalidUri(crypto::PublicError),

    #[error("Signature has an invalid length. Read {read} bytes, expected {expected} bytes")]
    SignatureInvalidLength {
        /// Amount of signature bytes read.
        read: usize,
        /// Expected number of signature bytes.
        expected: usize,
    },

    #[error("Unknown key type, must be a known 4-character sequence")]
    KeyTypeInvalid,

    #[error("Signature verification failed")]
    SignatureInvalid,

    #[error("Key store operation failed")]
    KeyStoreOperation,

    #[error("Key storage issue encountered")]
    KeyStorage(#[from] sc_keystore::Error),

    #[error("Invalid hexadecimal string data")]
    HexDataConversion(#[from] hex::FromHexError),

    /// Application specific error chain sequence forwarder.
    #[error(transparent)]
    Application(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),

    #[error(transparent)]
    GlobalLoggerError(#[from] sc_tracing::logging::Error),
}

impl std::convert::From<&str> for Error {
    fn from(s: &str) -> Error {
        Error::Input(s.to_string())
    }
}

impl std::convert::From<String> for Error {
    fn from(s: String) -> Error {
        Error::Input(s)
    }
}

impl std::convert::From<crypto::PublicError> for Error {
    fn from(e: crypto::PublicError) -> Error {
        Error::InvalidUri(e)
    }
}
