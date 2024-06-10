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

use crate::{
    globals::InstanceAccessGlobal,
    lazy_pages::{self, FuzzerLazyPagesContext},
    print_module, Runner, ENV, MEMORY_BYTES, PROGRAM_GAS,
};
use anyhow::Result;
use gear_wasm_gen::SyscallName;
use gear_wasm_instrument::{parity_wasm::elements::Module, GLOBAL_NAME_GAS};
use wasmer::{
    Exports, Extern, Function, ImportObject, Instance, Memory, MemoryType, Module as WasmerModule,
    Store, Val,
};

impl InstanceAccessGlobal for Instance {
    fn set_global(&mut self, name: &str, value: i64) -> Result<()> {
        let global = self.exports.get_global(name)?;
        global.set(Val::I64(value))?;
        Ok(())
    }

    fn get_global(&mut self, name: &str) -> Result<i64> {
        let global = self.exports.get_global(name)?;
        if let Val::I64(v) = global.get() {
            Ok(v)
        } else {
            Err(anyhow::anyhow!("Global is not an i64"))
        }
    }
}

fn gr_system_break(_code: i32) {
    log::error!("system break trap");
}

pub struct WasmerRunner;

impl Runner for WasmerRunner {
    fn run(module: &Module) -> Result<()> {
        let store = Store::default();
        let wasmer_module =
            WasmerModule::new(&store, module.clone().into_bytes().expect("valid bytes"))?;

        let ty = MemoryType::new(MEMORY_BYTES, None, false);
        let m = Memory::new(&store, ty).expect("memory allocated");
        let data_ptr = m.data_ptr() as usize;
        let memory = Extern::Memory(m);

        let mem_range = data_ptr..(data_ptr + MEMORY_BYTES as usize);

        let mut exports = Exports::new();
        exports.insert("memory".to_string(), memory.clone());

        exports.insert(
            SyscallName::SystemBreak.to_str(),
            Extern::Function(Function::new_native(&store, gr_system_break)),
        );

        let mut imports = ImportObject::new();
        imports.register(ENV, exports);

        let instance = match Instance::new(&wasmer_module, &imports) {
            Ok(instance) => instance,
            Err(e) => {
                print_module(module);
                panic!("Failed to instantiate module: {:?}", e);
            }
        };

        lazy_pages::init_fuzzer_lazy_pages(FuzzerLazyPagesContext {
            instance: Box::new(instance.clone()),
            range: mem_range,
        });

        let gear_gas = instance
            .exports
            .get_global(GLOBAL_NAME_GAS)
            .expect("global exists");
        gear_gas.set(Val::I64(PROGRAM_GAS)).expect("global set");

        print_module(module);

        let init_fn = instance
            .exports
            .get_function("init")
            .expect("init function");
        init_fn.call(&[]).expect("init function call");

        Ok(())
    }
}
