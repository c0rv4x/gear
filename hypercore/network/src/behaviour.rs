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

use crate::peer_info;
use libp2p::{
    core::Multiaddr, identify::Info as IdentifyInfo, identity::PublicKey, swarm::NetworkBehaviour,
    PeerId,
};

use parking_lot::Mutex;
use std::{collections::HashSet, sync::Arc};

use crate::error;

/// General behaviour of the network. Combines all protocols together.
#[derive(NetworkBehaviour)]
#[behaviour(to_swarm = "BehaviourOut")]
pub struct Behaviour {
    /// Periodically pings and identifies the nodes we are connected to, and store information in a
    /// cache.
    peer_info: peer_info::PeerInfoBehaviour,
    /// Keep track of active and pending connections to enforce hard limits.
    connection_limits: libp2p::connection_limits::Behaviour,
}

/// Event generated by `Behaviour`.
pub enum BehaviourOut {
    /// We have obtained identity information from a peer, including the addresses it is listening
    /// on.
    PeerIdentify {
        /// Id of the peer that has been identified.
        peer_id: PeerId,
        /// Information about the peer.
        info: Box<IdentifyInfo>,
    },

    /// Ignored event generated by lower layers.
    None,
}

impl Behaviour {
    /// Builds a new `Behaviour`.
    pub fn new(
        user_agent: String,
        local_public_key: PublicKey,
        external_addresses: Arc<Mutex<HashSet<Multiaddr>>>,
        connection_limits: libp2p::connection_limits::Behaviour,
    ) -> Result<Self, error::Error> {
        Ok(Self {
            peer_info: peer_info::PeerInfoBehaviour::new(
                user_agent,
                local_public_key,
                external_addresses,
            ),
            connection_limits,
        })
    }

    /// Borrows `self` and returns a struct giving access to the information about a node.
    ///
    /// Returns `None` if we don't know anything about this node. Always returns `Some` for nodes
    /// we're connected to, meaning that if `None` is returned then we're not connected to that
    /// node.
    pub fn node(&self, peer_id: &PeerId) -> Option<peer_info::Node> {
        self.peer_info.node(peer_id)
    }
}

impl From<void::Void> for BehaviourOut {
    fn from(_event: void::Void) -> Self {
        Self::None
    }
}

impl From<peer_info::PeerInfoEvent> for BehaviourOut {
    fn from(event: peer_info::PeerInfoEvent) -> Self {
        let peer_info::PeerInfoEvent::Identified { peer_id, info } = event;
        BehaviourOut::PeerIdentify {
            peer_id,
            info: Box::new(info),
        }
    }
}
