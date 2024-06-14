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

use crate::*;
use gear_core::{
    ids::prelude::CodeIdExt,
    message::{DispatchKind, Payload},
    program::ProgramState as InitStatus,
};
use gprimitives::{ActorId, MessageId};
use hypercore_db::{BlockInfo, MemDb};
use hypercore_runtime_common::state::{self, Dispatch, MaybeHash, ProgramState, Storage};
use std::collections::{BTreeMap, VecDeque};
use utils::*;
use wabt::wat2wasm;

fn init_new_block(processor: &mut Processor, height: u32, timestamp: u64) {
    let chain_head = H256::random();
    processor
        .db
        .set_block_info(chain_head, BlockInfo { height, timestamp });
    processor.creator.set_chain_head(chain_head);
}

#[test]
fn handle_new_code_valid() {
    init_logger();

    let db = MemDb::default();
    let mut processor =
        Processor::new(Database::from_one(&db)).expect("failed to create processor");

    init_new_block(&mut processor, 0, 0);

    let (code_id, original_code) = utils::wat_to_wasm(utils::VALID_PROGRAM);
    let original_code_len = original_code.len();

    assert!(processor.db.read_original_code(code_id).is_none());
    assert!(processor
        .db
        .read_instrumented_code(hypercore_runtime::VERSION, code_id)
        .is_none());

    let calculated_id = processor
        .handle_new_code(original_code.clone())
        .expect("failed to call runtime api")
        .expect("code failed verification or instrumentation");

    assert_eq!(calculated_id, code_id);

    assert_eq!(
        processor
            .db
            .read_original_code(code_id)
            .expect("failed to read original code"),
        original_code
    );
    assert!(
        processor
            .db
            .read_instrumented_code(hypercore_runtime::VERSION, code_id)
            .expect("failed to read original code")
            .code()
            .len()
            > original_code_len
    );
}

#[test]
fn handle_new_code_invalid() {
    init_logger();

    let db = MemDb::default();
    let mut processor =
        Processor::new(Database::from_one(&db)).expect("failed to create processor");

    init_new_block(&mut processor, 0, 0);

    let (code_id, original_code) = utils::wat_to_wasm(utils::INVALID_PROGRAM);

    assert!(processor.db.read_original_code(code_id).is_none());
    assert!(processor
        .db
        .read_instrumented_code(hypercore_runtime::VERSION, code_id)
        .is_none());

    assert!(processor
        .handle_new_code(original_code.clone())
        .expect("failed to call runtime api")
        .is_none());

    assert!(processor.db.read_original_code(code_id).is_none());
    assert!(processor
        .db
        .read_instrumented_code(hypercore_runtime::VERSION, code_id)
        .is_none());
}

#[test]
fn host_ping_pong() {
    init_logger();

    let db = MemDb::default();
    let mut processor = Processor::new(Database::from_one(&db)).unwrap();

    init_new_block(&mut processor, 0, 0);

    let program_id = 42.into();

    let code_id = processor
        .handle_new_code(demo_ping::WASM_BINARY)
        .expect("failed to call runtime api")
        .expect("code failed verification or instrumentation");

    let state_hash = create_program(
        &mut processor,
        program_id,
        code_id,
        create_message(DispatchKind::Init, "PING"),
    )
    .unwrap();

    let _init = processor.run_on_host(program_id, state_hash).unwrap();
}

#[test]
fn ping_pong() {
    init_logger();

    let db = MemDb::default();
    let mut processor = Processor::new(Database::from_one(&db)).unwrap();

    init_new_block(&mut processor, 0, 0);

    let user_id = ActorId::from(10);
    let program_id = ProgramId::from(0x10000);

    let code_id = processor
        .handle_new_code(demo_ping::WASM_BINARY)
        .expect("failed to call runtime api")
        .expect("code failed verification or instrumentation");

    let state_hash = create_program(
        &mut processor,
        program_id,
        code_id,
        UserMessage {
            id: MessageId::from(1),
            kind: DispatchKind::Init,
            source: user_id,
            payload: b"PING".to_vec(),
            gas_limit: 1_000_000_000,
            value: 1,
        },
    )
    .unwrap();

    let mut programs = BTreeMap::from_iter([(program_id, state_hash)]);

    send_user_message(
        &mut processor,
        &mut programs,
        program_id,
        UserMessage {
            id: MessageId::from(2),
            kind: DispatchKind::Handle,
            source: user_id,
            payload: b"PING".to_vec(),
            gas_limit: 1_000_000_000,
            value: 1,
        },
    );

    let to_users = run::run(8, processor.creator.clone(), &mut programs);

    assert_eq!(to_users.len(), 2);

    let message = &to_users[0];
    assert_eq!(message.destination(), user_id);
    assert_eq!(message.payload_bytes(), b"PONG");

    let message = &to_users[1];
    assert_eq!(message.destination(), user_id);
    assert_eq!(message.payload_bytes(), b"PONG");
}

fn create_message(kind: DispatchKind, payload: impl AsRef<[u8]>) -> UserMessage {
    UserMessage {
        id: H256::random().0.into(),
        kind,
        source: H256::random().0.into(),
        payload: payload.as_ref().to_vec(),
        gas_limit: 1_000_000_000,
        value: 0,
    }
}

fn create_program(
    processor: &mut Processor,
    program_id: ProgramId,
    code_id: CodeId,
    init_message: UserMessage,
) -> Result<H256> {
    assert_eq!(init_message.kind, DispatchKind::Init);

    processor.db.set_program_code_id(program_id, code_id);

    let payload_hash = match init_message.payload.len() {
        0 => MaybeHash::Empty,
        _ => processor
            .db
            .write_payload(Payload::try_from(init_message.payload.clone()).unwrap())
            .into(),
    };

    let init_dispatch = Dispatch {
        id: init_message.id,
        kind: DispatchKind::Init,
        source: init_message.source,
        payload_hash,
        gas_limit: init_message.gas_limit,
        value: init_message.value,
        details: None,
        context: None,
    };

    let queue = VecDeque::from(vec![init_dispatch]);
    let queue_hash = processor.db.write_queue(queue);

    let active_program = state::ActiveProgram {
        allocations_hash: MaybeHash::Empty,
        pages_hash: MaybeHash::Empty,
        gas_reservation_map_hash: MaybeHash::Empty,
        memory_infix: Default::default(),
        status: InitStatus::Uninitialized {
            message_id: init_message.id,
        },
    };

    let program_state = ProgramState {
        state: state::Program::Active(active_program),
        queue_hash: queue_hash.into(),
        waitlist_hash: MaybeHash::Empty,
        balance: 0,
    };

    Ok(processor.db.write_state(program_state))
}

fn send_user_message(
    processor: &mut Processor,
    programs: &mut BTreeMap<ProgramId, H256>,
    program_id: ProgramId,
    message: UserMessage,
) {
    let payload_hash = match message.payload.len() {
        0 => MaybeHash::Empty,
        _ => processor
            .db
            .write_payload(Payload::try_from(message.payload).unwrap())
            .into(),
    };

    let dispatch = Dispatch {
        id: message.id,
        kind: message.kind,
        source: message.source,
        payload_hash,
        gas_limit: message.gas_limit,
        value: message.value,
        details: None,
        context: None,
    };

    let mut program_state = processor.db.read_state(programs[&program_id]).unwrap();
    let mut queue = program_state
        .queue_hash
        .with_hash_or_default(|hash| processor.db.read_queue(hash).unwrap());
    queue.push_back(dispatch);
    log::info!("Process queue after send message: {queue:?}");
    let queue_hash = processor.db.write_queue(queue);
    program_state.queue_hash = queue_hash.into();
    let new_state_hash = processor.db.write_state(program_state);
    programs.insert(program_id, new_state_hash);
}

#[test]
fn async_and_ping() {
    init_logger();

    let mut message_nonce: u64 = 0;
    let mut get_next_message_id = || {
        message_nonce += 1;
        MessageId::from(message_nonce)
    };
    let user_id = ActorId::from(10);

    let db = MemDb::default();
    let mut processor = Processor::new(Database::from_one(&db)).unwrap();

    init_new_block(&mut processor, 0, 0);

    let ping_id = ProgramId::from(0x10000000);
    let async_id = ProgramId::from(0x20000000);

    let ping_code_id = processor
        .handle_new_code(demo_ping::WASM_BINARY)
        .expect("failed to call runtime api")
        .expect("code failed verification or instrumentation");

    let upload_code_id = processor
        .handle_new_code(demo_async::WASM_BINARY)
        .expect("failed to call runtime api")
        .expect("code failed verification or instrumentation");

    let ping_state_hash = create_program(
        &mut processor,
        ping_id,
        ping_code_id,
        UserMessage {
            id: get_next_message_id(),
            kind: DispatchKind::Init,
            source: user_id,
            payload: b"PING".to_vec(),
            gas_limit: 1_000_000_000,
            value: 0,
        },
    )
    .unwrap();

    let async_state_hash = create_program(
        &mut processor,
        async_id,
        upload_code_id,
        UserMessage {
            id: get_next_message_id(),
            kind: DispatchKind::Init,
            source: user_id,
            payload: ping_id.encode(),
            gas_limit: 1_000_000_000,
            value: 0,
        },
    )
    .unwrap();

    let mut programs = vec![(ping_id, ping_state_hash), (async_id, async_state_hash)]
        .into_iter()
        .collect::<BTreeMap<_, _>>();

    let wait_for_reply_to = get_next_message_id();
    send_user_message(
        &mut processor,
        &mut programs,
        async_id,
        UserMessage {
            id: wait_for_reply_to,
            kind: DispatchKind::Handle,
            source: user_id,
            payload: demo_async::Command::Common.encode(),
            gas_limit: 10_000_000_000,
            value: 0,
        },
    );

    let to_users = run::run(8, processor.creator.clone(), &mut programs);

    assert_eq!(to_users.len(), 3);

    let message = &to_users[0];
    assert_eq!(message.destination(), user_id);
    assert_eq!(message.payload_bytes(), b"PONG");

    let message = &to_users[1];
    assert_eq!(message.destination(), user_id);
    assert_eq!(message.payload_bytes(), b"");

    let message = &to_users[2];
    assert_eq!(message.destination(), user_id);
    assert_eq!(
        message.payload_bytes(),
        wait_for_reply_to.into_bytes().as_slice()
    );
}

#[test]
fn many_waits() {
    init_logger();

    let threads_amount = 8;

    let wat = r#"
        (module
            (import "env" "memory" (memory 1))
            (import "env" "gr_reply" (func $reply (param i32 i32 i32 i32)))
            (import "env" "gr_wait_for" (func $wait_for (param i32)))
            (export "handle" (func $handle))
            (func $handle
                (if
                    (i32.eqz (i32.load (i32.const 0x200)))
                    (then
                        (i32.store (i32.const 0x200) (i32.const 1))
                        (call $wait_for (i32.const 10))
                    )
                    (else
                        (call $reply (i32.const 0) (i32.const 13) (i32.const 0x400) (i32.const 0x600))
                    )
                )
            )
            (data (i32.const 0) "Hello, world!")
        )
    "#;

    let (_, code) = wat_to_wasm(wat);

    let db = MemDb::default();
    let mut processor = Processor::new(Database::from_one(&db)).unwrap();

    init_new_block(&mut processor, 0, 0);

    let code_id = processor
        .handle_new_code(code)
        .expect("failed to call runtime api")
        .expect("code failed verification or instrumentation");

    let amount = 10000;
    let mut programs = BTreeMap::new();
    for i in 0..amount {
        let program_id = ProgramId::from(i);
        let state_hash = create_program(
            &mut processor,
            program_id,
            code_id,
            create_message(DispatchKind::Init, b""),
        )
        .unwrap();

        programs.insert(program_id, state_hash);
    }

    let to_users = run::run(threads_amount, processor.creator.clone(), &mut programs);
    assert_eq!(to_users.len(), amount as usize);

    for &program_id in programs.clone().keys() {
        send_user_message(
            &mut processor,
            &mut programs,
            program_id,
            create_message(DispatchKind::Handle, b""),
        );
    }

    let to_users = run::run(threads_amount, processor.creator.clone(), &mut programs);
    assert_eq!(to_users.len(), 0);

    init_new_block(&mut processor, 11, 11);

    let to_users = run::run(threads_amount, processor.creator.clone(), &mut programs);

    assert_eq!(to_users.len(), amount as usize);

    for message in to_users {
        assert_eq!(message.payload_bytes(), b"Hello, world!");
    }
}

mod utils {
    use super::*;

    pub const VALID_PROGRAM: &str = r#"
        (module
        (import "env" "memory" (memory 1))
        (import "env" "gr_reply" (func $reply (param i32 i32 i32 i32)))
        (export "init" (func $init))
        (func $init
            (call $reply (i32.const 0) (i32.const 32) (i32.const 222) (i32.const 333))
        )
    )"#;

    pub const INVALID_PROGRAM: &str = r#"
        (module
        (import "env" "world" (func $world))
        (export "hello" (func $hello))
        (func $hello
            (call $world)
        )
    )"#;

    pub fn init_logger() {
        let _ = env_logger::Builder::from_default_env()
            .format_module_path(false)
            .format_level(true)
            .try_init();
    }

    pub fn wat_to_wasm(wat: &str) -> (CodeId, Vec<u8>) {
        let code = wat2wasm(wat).expect("failed to parse wat to bin");
        let code_id = CodeId::generate(&code);

        (code_id, code)
    }
}
