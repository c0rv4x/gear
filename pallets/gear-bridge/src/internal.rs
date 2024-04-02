// This file is part of Gear.

// Copyright (C) 2024 Gear Technologies Inc.
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

use crate::Hasher;
use frame_support::traits::Get;
use gear_core::message::Payload;
use parity_scale_codec::{Decode, Encode};
use primitive_types::{H160, H256, U256};
use scale_info::TypeInfo;
use sp_runtime::traits::Hash;

/// `OnEmpty` implementation for `Nonce` storage.
pub(crate) struct FirstNonce;

impl Get<U256> for FirstNonce {
    fn get() -> U256 {
        U256::one()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Decode, Encode)]
pub struct EthMessageData {
    destination: H160,
    payload: Payload,
}

impl EthMessageData {
    pub fn new(destination: H160, payload: Payload) -> Self {
        Self {
            destination,
            payload,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Decode, Encode, TypeInfo)]
pub struct EthMessage {
    source: H256,
    destination: H160,
    payload: Vec<u8>,
    nonce: U256,
}

impl EthMessage {
    pub(crate) fn from_data(source: H256, data: EthMessageData, nonce: U256) -> Self {
        let EthMessageData {
            destination,
            payload,
        } = data;
        let payload = payload.into_vec();

        Self {
            source,
            destination,
            payload,
            nonce,
        }
    }

    pub fn hash(&self) -> H256 {
        let mut nonce = [0; 32];

        self.nonce.to_little_endian(&mut nonce);

        let arg = [
            self.source.as_bytes(),
            self.destination.as_bytes(),
            self.payload.as_ref(),
            nonce.as_ref(),
        ]
        .concat();

        Hasher::hash(&arg)
    }
}
