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

//! Types that should only be used for testing!

use crate::{
    vrf::{make_transcript, VRFSignature, VRFTranscriptData},
    {CryptoStore, Error, SyncCryptoStore, SyncCryptoStorePtr},
};
use async_trait::async_trait;
use parking_lot::RwLock;
use sp_core::crypto::KeyTypeId;
use sp_core::{
    crypto::{CryptoTypePublicPair, Pair, Public},
    ecdsa, ed25519, redjubjub, sr25519,
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

/// A keystore implementation usable in tests.
#[derive(Default)]
pub struct KeyStore {
    /// `KeyTypeId` maps to public keys and public keys map to private keys.
    keys: Arc<RwLock<HashMap<KeyTypeId, HashMap<Vec<u8>, String>>>>,
}

impl KeyStore {
    /// Creates a new instance of `Self`.
    pub fn new() -> Self {
        Self::default()
    }

    fn sr25519_key_pair(&self, id: KeyTypeId, pub_key: &sr25519::Public) -> Option<sr25519::Pair> {
        self.keys.read().get(&id).and_then(|inner| {
            inner.get(pub_key.as_slice()).map(|s| {
                sr25519::Pair::from_string(s, None).expect("`sr25519` seed slice is valid")
            })
        })
    }

    fn ed25519_key_pair(&self, id: KeyTypeId, pub_key: &ed25519::Public) -> Option<ed25519::Pair> {
        self.keys.read().get(&id).and_then(|inner| {
            inner.get(pub_key.as_slice()).map(|s| {
                ed25519::Pair::from_string(s, None).expect("`ed25519` seed slice is valid")
            })
        })
    }

    fn ecdsa_key_pair(&self, id: KeyTypeId, pub_key: &ecdsa::Public) -> Option<ecdsa::Pair> {
        self.keys.read().get(&id).and_then(|inner| {
            inner
                .get(pub_key.as_slice())
                .map(|s| ecdsa::Pair::from_string(s, None).expect("`ecdsa` seed slice is valid"))
        })
    }

    fn redjubjub_key_pair(
        &self,
        id: KeyTypeId,
        pub_key: &redjubjub::Public,
    ) -> Option<redjubjub::Pair> {
        self.keys.read().get(&id).and_then(|inner| {
            inner.get(pub_key.as_slice()).map(|s| {
                redjubjub::Pair::from_string(s, None).expect("`redjubjub` seed slice is valid")
            })
        })
    }
}

#[async_trait]
impl CryptoStore for KeyStore {
    async fn keys(&self, id: KeyTypeId) -> Result<Vec<CryptoTypePublicPair>, Error> {
        SyncCryptoStore::keys(self, id)
    }

    async fn sr25519_public_keys(&self, id: KeyTypeId) -> Vec<sr25519::Public> {
        SyncCryptoStore::sr25519_public_keys(self, id)
    }

    async fn sr25519_generate_new(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<sr25519::Public, Error> {
        SyncCryptoStore::sr25519_generate_new(self, id, seed)
    }

    async fn ed25519_public_keys(&self, id: KeyTypeId) -> Vec<ed25519::Public> {
        SyncCryptoStore::ed25519_public_keys(self, id)
    }

    async fn ed25519_generate_new(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<ed25519::Public, Error> {
        SyncCryptoStore::ed25519_generate_new(self, id, seed)
    }

    async fn ecdsa_public_keys(&self, id: KeyTypeId) -> Vec<ecdsa::Public> {
        SyncCryptoStore::ecdsa_public_keys(self, id)
    }

    async fn ecdsa_generate_new(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<ecdsa::Public, Error> {
        SyncCryptoStore::ecdsa_generate_new(self, id, seed)
    }

    async fn redjubjub_public_keys(&self, id: KeyTypeId) -> Vec<redjubjub::Public> {
        SyncCryptoStore::redjubjub_public_keys(self, id)
    }

    async fn redjubjub_generate_new(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<redjubjub::Public, Error> {
        SyncCryptoStore::redjubjub_generate_new(self, id, seed)
    }

    async fn insert_unknown(&self, id: KeyTypeId, suri: &str, public: &[u8]) -> Result<(), ()> {
        SyncCryptoStore::insert_unknown(self, id, suri, public)
    }

    async fn has_keys(&self, public_keys: &[(Vec<u8>, KeyTypeId)]) -> bool {
        SyncCryptoStore::has_keys(self, public_keys)
    }

    async fn supported_keys(
        &self,
        id: KeyTypeId,
        keys: Vec<CryptoTypePublicPair>,
    ) -> std::result::Result<Vec<CryptoTypePublicPair>, Error> {
        SyncCryptoStore::supported_keys(self, id, keys)
    }

    async fn sign_with(
        &self,
        id: KeyTypeId,
        key: &CryptoTypePublicPair,
        msg: &[u8],
    ) -> Result<Vec<u8>, Error> {
        SyncCryptoStore::sign_with(self, id, key, msg)
    }

    async fn sr25519_vrf_sign(
        &self,
        key_type: KeyTypeId,
        public: &sr25519::Public,
        transcript_data: VRFTranscriptData,
    ) -> Result<VRFSignature, Error> {
        SyncCryptoStore::sr25519_vrf_sign(self, key_type, public, transcript_data)
    }
}

impl SyncCryptoStore for KeyStore {
    fn keys(&self, id: KeyTypeId) -> Result<Vec<CryptoTypePublicPair>, Error> {
        self.keys
            .read()
            .get(&id)
            .map(|map| {
                Ok(map.keys().fold(Vec::new(), |mut v, k| {
                    v.push(CryptoTypePublicPair(sr25519::CRYPTO_ID, k.clone()));
                    v.push(CryptoTypePublicPair(ed25519::CRYPTO_ID, k.clone()));
                    v.push(CryptoTypePublicPair(ecdsa::CRYPTO_ID, k.clone()));
                    v.push(CryptoTypePublicPair(redjubjub::CRYPTO_ID, k.clone()));
                    v
                }))
            })
            .unwrap_or_else(|| Ok(vec![]))
    }

    fn sr25519_public_keys(&self, id: KeyTypeId) -> Vec<sr25519::Public> {
        self.keys
            .read()
            .get(&id)
            .map(|keys| {
                keys.values()
                    .map(|s| {
                        sr25519::Pair::from_string(s, None).expect("`sr25519` seed slice is valid")
                    })
                    .map(|p| p.public())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn sr25519_generate_new(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<sr25519::Public, Error> {
        match seed {
            Some(seed) => {
                let pair = sr25519::Pair::from_string(seed, None).map_err(|_| {
                    Error::ValidationError("Generates an `sr25519` pair.".to_owned())
                })?;
                self.keys
                    .write()
                    .entry(id)
                    .or_default()
                    .insert(pair.public().to_raw_vec(), seed.into());
                Ok(pair.public())
            }
            None => {
                let (pair, phrase, _) = sr25519::Pair::generate_with_phrase(None);
                self.keys
                    .write()
                    .entry(id)
                    .or_default()
                    .insert(pair.public().to_raw_vec(), phrase);
                Ok(pair.public())
            }
        }
    }

    fn ed25519_public_keys(&self, id: KeyTypeId) -> Vec<ed25519::Public> {
        self.keys
            .read()
            .get(&id)
            .map(|keys| {
                keys.values()
                    .map(|s| {
                        ed25519::Pair::from_string(s, None).expect("`ed25519` seed slice is valid")
                    })
                    .map(|p| p.public())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn ed25519_generate_new(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<ed25519::Public, Error> {
        match seed {
            Some(seed) => {
                let pair = ed25519::Pair::from_string(seed, None).map_err(|_| {
                    Error::ValidationError("Generates an `ed25519` pair.".to_owned())
                })?;
                self.keys
                    .write()
                    .entry(id)
                    .or_default()
                    .insert(pair.public().to_raw_vec(), seed.into());
                Ok(pair.public())
            }
            None => {
                let (pair, phrase, _) = ed25519::Pair::generate_with_phrase(None);
                self.keys
                    .write()
                    .entry(id)
                    .or_default()
                    .insert(pair.public().to_raw_vec(), phrase);
                Ok(pair.public())
            }
        }
    }

    fn ecdsa_public_keys(&self, id: KeyTypeId) -> Vec<ecdsa::Public> {
        self.keys
            .read()
            .get(&id)
            .map(|keys| {
                keys.values()
                    .map(|s| {
                        ecdsa::Pair::from_string(s, None).expect("`ecdsa` seed slice is valid")
                    })
                    .map(|p| p.public())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn ecdsa_generate_new(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<ecdsa::Public, Error> {
        match seed {
            Some(seed) => {
                let pair = ecdsa::Pair::from_string(seed, None)
                    .map_err(|_| Error::ValidationError("Generates an `ecdsa` pair.".to_owned()))?;
                self.keys
                    .write()
                    .entry(id)
                    .or_default()
                    .insert(pair.public().to_raw_vec(), seed.into());
                Ok(pair.public())
            }
            None => {
                let (pair, phrase, _) = ecdsa::Pair::generate_with_phrase(None);
                self.keys
                    .write()
                    .entry(id)
                    .or_default()
                    .insert(pair.public().to_raw_vec(), phrase);
                Ok(pair.public())
            }
        }
    }

    fn redjubjub_public_keys(&self, id: KeyTypeId) -> Vec<redjubjub::Public> {
        self.keys
            .read()
            .get(&id)
            .map(|keys| {
                keys.values()
                    .map(|s| {
                        redjubjub::Pair::from_string(s, None)
                            .expect("`redjubjub` seed slice is valid")
                    })
                    .map(|p| p.public())
                    .collect()
            })
            .unwrap_or_default()
    }

    fn redjubjub_generate_new(
        &self,
        id: KeyTypeId,
        seed: Option<&str>,
    ) -> Result<redjubjub::Public, Error> {
        match seed {
            Some(seed) => {
                let pair = redjubjub::Pair::from_string(seed, None).map_err(|_| {
                    Error::ValidationError("Generates an `redjubjub` pair.".to_owned())
                })?;
                self.keys
                    .write()
                    .entry(id)
                    .or_default()
                    .insert(pair.public().to_raw_vec(), seed.into());
                Ok(pair.public())
            }
            None => {
                let (pair, phrase, _) = redjubjub::Pair::generate_with_phrase(None);
                self.keys
                    .write()
                    .entry(id)
                    .or_default()
                    .insert(pair.public().to_raw_vec(), phrase);
                Ok(pair.public())
            }
        }
    }

    fn insert_unknown(&self, id: KeyTypeId, suri: &str, public: &[u8]) -> Result<(), ()> {
        self.keys
            .write()
            .entry(id)
            .or_default()
            .insert(public.to_owned(), suri.to_string());
        Ok(())
    }

    fn has_keys(&self, public_keys: &[(Vec<u8>, KeyTypeId)]) -> bool {
        public_keys
            .iter()
            .all(|(k, t)| self.keys.read().get(&t).and_then(|s| s.get(k)).is_some())
    }

    fn supported_keys(
        &self,
        id: KeyTypeId,
        keys: Vec<CryptoTypePublicPair>,
    ) -> std::result::Result<Vec<CryptoTypePublicPair>, Error> {
        let provided_keys = keys.into_iter().collect::<HashSet<_>>();
        let all_keys = SyncCryptoStore::keys(self, id)?
            .into_iter()
            .collect::<HashSet<_>>();

        Ok(provided_keys.intersection(&all_keys).cloned().collect())
    }

    fn sign_with(
        &self,
        id: KeyTypeId,
        key: &CryptoTypePublicPair,
        msg: &[u8],
    ) -> Result<Vec<u8>, Error> {
        use codec::Encode;

        match key.0 {
            ed25519::CRYPTO_ID => {
                let key_pair: ed25519::Pair = self
                    .ed25519_key_pair(id, &ed25519::Public::from_slice(key.1.as_slice()))
                    .ok_or_else(|| Error::PairNotFound("ed25519".to_owned()))?;
                return Ok(key_pair.sign(msg).encode());
            }
            sr25519::CRYPTO_ID => {
                let key_pair: sr25519::Pair = self
                    .sr25519_key_pair(id, &sr25519::Public::from_slice(key.1.as_slice()))
                    .ok_or_else(|| Error::PairNotFound("sr25519".to_owned()))?;
                return Ok(key_pair.sign(msg).encode());
            }
            ecdsa::CRYPTO_ID => {
                let key_pair: ecdsa::Pair = self
                    .ecdsa_key_pair(id, &ecdsa::Public::from_slice(key.1.as_slice()))
                    .ok_or_else(|| Error::PairNotFound("ecdsa".to_owned()))?;
                return Ok(key_pair.sign(msg).encode());
            }
            redjubjub::CRYPTO_ID => {
                let key_pair: redjubjub::Pair = self
                    .redjubjub_key_pair(id, &redjubjub::Public::from_slice(key.1.as_slice()))
                    .ok_or_else(|| Error::PairNotFound("redjubjub".to_owned()))?;
                return Ok(key_pair.sign(msg).encode());
            }
            _ => Err(Error::KeyNotSupported(id)),
        }
    }

    fn sr25519_vrf_sign(
        &self,
        key_type: KeyTypeId,
        public: &sr25519::Public,
        transcript_data: VRFTranscriptData,
    ) -> Result<VRFSignature, Error> {
        let transcript = make_transcript(transcript_data);
        let pair = self
            .sr25519_key_pair(key_type, public)
            .ok_or_else(|| Error::PairNotFound("Not found".to_owned()))?;
        let (inout, proof, _) = pair.as_ref().vrf_sign(transcript);
        Ok(VRFSignature {
            output: inout.to_output(),
            proof,
        })
    }
}

impl Into<SyncCryptoStorePtr> for KeyStore {
    fn into(self) -> SyncCryptoStorePtr {
        Arc::new(self)
    }
}

impl Into<Arc<dyn CryptoStore>> for KeyStore {
    fn into(self) -> Arc<dyn CryptoStore> {
        Arc::new(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{vrf::VRFTranscriptValue, SyncCryptoStore};
    use sp_core::{
        sr25519,
        testing::{ED25519, REDJUBJUB, SR25519},
    };

    #[test]
    fn store_key_and_extract() {
        let store = KeyStore::new();

        let public =
            SyncCryptoStore::ed25519_generate_new(&store, ED25519, None).expect("Generates key");

        let public_keys = SyncCryptoStore::keys(&store, ED25519).unwrap();

        assert!(public_keys.contains(&public.into()));
    }

    #[test]
    fn store_unknown_and_extract_it() {
        let store = KeyStore::new();

        let secret_uri = "//Alice";
        let key_pair = sr25519::Pair::from_string(secret_uri, None).expect("Generates key pair");

        SyncCryptoStore::insert_unknown(&store, SR25519, secret_uri, key_pair.public().as_ref())
            .expect("Inserts unknown key");

        let public_keys = SyncCryptoStore::keys(&store, SR25519).unwrap();

        assert!(public_keys.contains(&key_pair.public().into()));
    }

    #[test]
    fn store_redjubjub_unknown_and_extract_it() {
        let store = KeyStore::new();

        let secret_uri = "//Alice";
        let key_pair = redjubjub::Pair::from_string(secret_uri, None).expect("Generates key pair");

        SyncCryptoStore::insert_unknown(&store, REDJUBJUB, secret_uri, key_pair.public().as_ref())
            .expect("Inserts unknown key");

        let public_keys = SyncCryptoStore::keys(&store, REDJUBJUB).unwrap();

        assert!(public_keys.contains(&key_pair.public().into()));
    }

    #[test]
    fn vrf_sign() {
        let store = KeyStore::new();

        let secret_uri = "//Alice";
        let key_pair = sr25519::Pair::from_string(secret_uri, None).expect("Generates key pair");

        let transcript_data = VRFTranscriptData {
            label: b"Test",
            items: vec![
                ("one", VRFTranscriptValue::U64(1)),
                ("two", VRFTranscriptValue::U64(2)),
                (
                    "three",
                    VRFTranscriptValue::Bytes("test".as_bytes().to_vec()),
                ),
            ],
        };

        let result = SyncCryptoStore::sr25519_vrf_sign(
            &store,
            SR25519,
            &key_pair.public(),
            transcript_data.clone(),
        );
        assert!(result.is_err());

        SyncCryptoStore::insert_unknown(&store, SR25519, secret_uri, key_pair.public().as_ref())
            .expect("Inserts unknown key");

        let result =
            SyncCryptoStore::sr25519_vrf_sign(&store, SR25519, &key_pair.public(), transcript_data);

        assert!(result.is_ok());
    }
}
