// This file is part of Gear.

// Copyright (C) 2021-2023 Gear Technologies Inc.
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

//! Entities describing config of pointer writes that `wasm-gen`
//! includes into generated wasms. They're used to set values of some
//! syscall parameters that are passed as pointer to the data.

use arbitrary::{Result, Unstructured};
use gear_wasm_instrument::syscalls::PtrType;
use std::{collections::HashMap, mem::size_of, ops::RangeInclusive};

pub use gear_wasm_instrument::syscalls::ParamType;

/// Pointer writes config.
///
/// Determines what data should be written into the pointers of
/// particular type. For example, it can be configured to write
/// some data (generated in [`PointerWriteData::generate_data_to_write`])
/// into all the pointers of type `*const/*mut Value` among syscall
/// parameters.
///
/// # Note
///
/// This config will not work for [`PtrType::BufferStart`].
#[derive(Debug, Clone)]
pub struct PointerWritesConfig(HashMap<PtrType, Vec<PointerWrite>>);

impl Default for PointerWritesConfig {
    fn default() -> PointerWritesConfig {
        let value_write_data = PointerWriteData::U128(0..=100_000_000_000);

        const HASH_LEN: usize = size_of::<gsys::Hash>() / size_of::<i32>();

        PointerWritesConfig(
            [
                (
                    PtrType::Value,
                    vec![PointerWrite {
                        offset: 0,
                        data: value_write_data.clone(),
                    }],
                ),
                (
                    PtrType::HashWithValue,
                    vec![PointerWrite {
                        offset: HASH_LEN,
                        data: value_write_data.clone(),
                    }],
                ),
                (
                    PtrType::TwoHashesWithValue,
                    vec![PointerWrite {
                        offset: 2 * HASH_LEN,
                        data: value_write_data,
                    }],
                ),
            ]
            .into_iter()
            .collect(),
        )
    }
}

impl PointerWritesConfig {
    pub fn empty() -> PointerWritesConfig {
        PointerWritesConfig(HashMap::new())
    }

    /// Set the `PointerWrite`s for the specified pointer type.
    pub fn set_rule(&mut self, ptr_type: PtrType, pointer_writes: Vec<PointerWrite>) {
        if matches!(ptr_type, PtrType::BufferStart { .. }) {
            panic!("PtrType::BufferStart is not supported");
        }

        self.0.insert(ptr_type, pointer_writes);
    }

    /// Get the `PointerWrite`s for the specified pointer type.
    pub fn get_rule(&self, ptr_type: PtrType) -> Option<Vec<PointerWrite>> {
        self.0.get(&ptr_type).cloned()
    }
}

/// Single chunk of data being written into the pointer address
/// with specified offset.
///
/// # Note:
///
/// Offset is relative to the actual pointer address that's set
/// in syscall invoke instructions.
#[derive(Debug, Clone)]
pub struct PointerWrite {
    /// Relative to the pointer address that's set in syscall invoke
    /// instructions.
    pub offset: usize,
    pub data: PointerWriteData,
}

/// Range of values being written into the pointer address. The
/// actual data can be generated by calling
/// [`PointerWriteData::generate_data_to_write`].
#[derive(Debug, Clone)]
pub enum PointerWriteData {
    U128(RangeInclusive<u128>),
}

impl PointerWriteData {
    /// Get the actual data that should be written into the memory.
    pub fn generate_data_to_write(&self, unstructured: &mut Unstructured) -> Result<Vec<i32>> {
        match self {
            Self::U128(range) => {
                let value = unstructured.int_in_range(range.clone())?;
                Ok(value
                    .to_le_bytes()
                    .chunks(size_of::<u128>() / size_of::<i32>())
                    .map(|word_bytes| i32::from_le_bytes(word_bytes.try_into().unwrap()))
                    .collect())
            }
        }
    }
}
