// This file is part of Gear.

// Copyright (C) 2021-2024 Gear Technologies Inc.
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

//! Gear builtin primitives.

use crate::ActorId;

/// Seed for generating builtin actor ids
pub const SEED: [u8; 8] = *b"built/in";

/// Index of builtin library [`gbuiltin_bls318`]
pub const BLS12_381_ID: u64 = 1;

/// Actor ID of builtin library [`gbuiltin_bls318`]
pub const BLS12_381: ActorId = ActorId([
    107, 110, 41, 44, 56, 41, 69, 232, 11, 245, 26, 242, 186, 127, 233, 244, 88, 220, 255, 129,
    174, 96, 117, 196, 111, 144, 149, 225, 187, 236, 220, 55,
]);

/// Actor ID of builtin library [`gbuiltin_eth_bridge`]
pub const ETH_BRIDGE_ID: u64 = 2;

/// Actor ID of builtin library [`gbuiltin_eth_bridge`]
pub const ETH_BRIDGE: ActorId = ActorId([
    119, 246, 94, 241, 144, 225, 27, 254, 203, 143, 200, 151, 15, 211, 116, 158, 148, 190, 214,
    106, 35, 236, 47, 122, 54, 35, 231, 133, 208, 129, 103, 97,
]);

/// Actor ID of builtin library [`gbuiltin_staking`]
pub const STAKING_ID: u64 = 3;

/// Actor ID of builtin library [`gbuiltin_staking`]
pub const STAKING: ActorId = ActorId([
    242, 129, 108, 237, 11, 21, 116, 149, 149, 57, 45, 58, 24, 181, 162, 54, 61, 111, 239, 229,
    179, 182, 21, 55, 57, 242, 24, 21, 27, 122, 205, 191,
]);

/// Resolve actor id from the input index
pub fn to_actor_id(idx: u64) -> ActorId {
    match idx {
        BLS12_381_ID => BLS12_381,
        ETH_BRIDGE_ID => ETH_BRIDGE,
        STAKING_ID => STAKING,
        _ => panic!("Unsupported builtin id: {idx}"),
    }
}

#[cfg(feature = "codec")]
#[cfg(test)]
mod tests {
    use crate::builtin::{
        BLS12_381, BLS12_381_ID, ETH_BRIDGE, ETH_BRIDGE_ID, SEED, STAKING, STAKING_ID,
    };
    use blake2::{digest::typenum::U32, Blake2b, Digest};
    use parity_scale_codec::Encode;

    /// Blake2 hash
    fn hash(data: &[u8]) -> [u8; 32] {
        let mut ctx = Blake2b::<U32>::new();
        ctx.update(data);
        ctx.finalize().into()
    }

    #[test]
    fn actor_ids_matched() {
        assert_eq!(hash((SEED, BLS12_381_ID).encode().as_slice()), BLS12_381.0);
        assert_eq!(
            hash((SEED, ETH_BRIDGE_ID).encode().as_slice()),
            ETH_BRIDGE.0
        );
        assert_eq!(hash((SEED, STAKING_ID).encode().as_slice()), STAKING.0);
    }
}
