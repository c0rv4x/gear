// This file is part of Gear.
//
// Copyright (C) 2024 Gear Technologies Inc.
// SPDX-License-Identifier: GPL-3.0-or-later WITH Classpath-exception-2.0
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <https://www.gnu.org/licenses/>.

//! Ethereum state observer for Hypercore.

use anyhow::Result;
use gear_core::ids::ProgramId;
use hypercore_db::Message;
use primitive_types::H256;
use std::{collections::HashMap, time::Duration};

/// Ethereum state observer.
///
/// Generally, it should exist in single state and should not be cloned.
pub struct Observer {
    ethereum_rpc: String,
    db: Box<dyn hypercore_db::Database>,
}

#[derive(Debug)]
pub enum Event {
    /// New chain head is known.
    NewHead {
        hash: H256,
        programs: Vec<ProgramId>,
        // TODO: replace `Message` with `StoredDispatch` with gas from `gear-core`
        messages: HashMap<ProgramId, Vec<Message>>,
    },
    /// New code was submitted.
    NewCode { hash: H256, code: Vec<u8> },
}

impl Observer {
    pub fn new(ethereum_rpc: String, db: Box<dyn hypercore_db::Database>) -> Result<Self> {
        Ok(Self { ethereum_rpc, db })
    }

    pub fn listen(self) -> impl futures::Stream<Item = Event> {
        use futures::{stream::poll_fn, task::Poll};

        futures::stream::poll_fn(move |_| {
            let hash = rand::random::<[u8; 32]>().into();
            let program_id = rand::random::<[u8; 32]>().into();

            let res = Event::NewHead {
                hash,
                programs: vec![],
                messages: [(program_id, vec![])].into_iter().collect(),
            };

            Poll::Ready(Some(res))
        })
    }
}
