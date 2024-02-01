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

//! CLI implementation for gring.

#![cfg(feature = "cli")]

use crate::{ss58, Keyring, Keystore};
use anyhow::{anyhow, Result};
use clap::Parser;
use colored::{ColoredString, Colorize};
use schnorrkel::{PublicKey, Signature};
use std::{fs, path::PathBuf};

/// gring sub commands.
#[derive(Parser)]
pub enum Command {
    /// Generate a new key.
    New {
        /// The name of the key.
        name: String,
        /// The passphrase of the key.
        #[arg(short, long)]
        passphrase: String,
        /// If the key should be a vanity key.
        #[arg(short, long)]
        vanity: Option<String>,
    },
    /// List all keys in keystore.
    #[clap(visible_alias = "l")]
    List {
        /// If only list the primary key.
        #[arg(short, long)]
        primary: bool,
    },
    /// Use the provided key as primary key.
    Use {
        /// Set the key as the primary key.
        key: String,
    },
    /// Sign a message.
    Sign {
        /// The singning context.
        #[clap(short, long, default_value = "gring.vara")]
        ctx: String,
        /// The message to sign.
        message: String,
        /// the passphrase of the primary key.
        #[clap(short, long)]
        passphrase: String,
    },
    /// Verify a message.
    Verify {
        /// The singning context.
        #[clap(short, long, default_value = "gring.vara")]
        ctx: String,
        /// The signed to message.
        message: String,
        /// The signature to verify.
        signature: String,
        /// The address used in the verification, supports hex
        /// public key bytes and VARA ss58 address.
        ///
        /// NOTE: if not provided, the address of the primary
        /// key will be used.
        #[arg(short, long)]
        address: Option<String>,
    },
}

impl Command {
    /// The path of the keyring store.
    ///
    /// NOTE: This is currently not configurable.
    pub fn store() -> Result<PathBuf> {
        let app = env!("CARGO_PKG_NAME");
        let store = dirs::data_dir()
            .ok_or_else(|| anyhow!("Failed to locate app directory."))?
            .join(app);

        fs::create_dir_all(&store).map_err(|e| {
            tracing::error!("Failed to create keyring store at {store:?}");
            e
        })?;

        tracing::info!(
            "keyring store: {}",
            store.display().to_string().underline().dimmed()
        );
        Ok(store)
    }

    /// Run the command.
    pub fn run(self) -> Result<()> {
        let mut keyring = Keyring::load(Command::store()?)?;
        match self {
            Command::New {
                mut name,
                vanity,
                passphrase,
            } => {
                if name.len() > 16 {
                    return Err(anyhow!("Name must be less than 16 characters."));
                }

                let raw_name = name.clone();
                let path = {
                    let mut path = keyring.store.join(&name).with_extension("json");
                    let mut count = 0;
                    while path.exists() {
                        name = format!("{}-{}", &raw_name, count);
                        path = keyring.store.join(&name).with_extension("json");
                        count += 1;
                    }

                    path
                };

                if name != raw_name {
                    tracing::info!(
                        "Key {} exists, auto switching to {}",
                        raw_name.underline(),
                        name.underline().cyan()
                    );
                }

                let (keystore, keypair) =
                    keyring.create(&name, vanity.as_deref(), Some(passphrase.as_ref()))?;

                println!("{:<16}{}", "Name:", name.bold());
                println!("{:<16}{}", "VARA Address: ", keystore.address);
                println!("{:<16}0x{}", "Public Key:", hex::encode(keypair.public));
                println!(
                    "Drag {} to the polkadot.js extension to import it.",
                    path.display().to_string().underline()
                );
            }
            Command::List { primary } => {
                let key = keyring.primary()?;
                if primary {
                    Self::print_key(&key);
                    return Ok(());
                }

                println!("| {:<16} | {:<49} |", "Name".bold(), "Address".bold());
                println!("| {} | {} |", "-".repeat(16), "-".repeat(49));

                for key in keyring.list() {
                    let mut name: ColoredString = key.meta.name.clone().into();
                    let mut address: ColoredString = key.address.clone().into();
                    if key.meta.name == keyring.primary {
                        name = name.cyan();
                        address = address.cyan();
                    };

                    println!("| {name:<16} | {address} |");
                }
            }
            Command::Use { key } => {
                let key = keyring.set_primary(key)?;
                println!("The primary key has been updated to:");
                Self::print_key(&key);
            }
            Command::Sign {
                ctx,
                message,
                passphrase,
            } => {
                let key = keyring.primary()?;
                let pair = key.decrypt_scrypt(passphrase.as_ref()).map_err(|e| {
                    anyhow!("Incorrect passphrase, failed to decrypt keystore, {e}")
                })?;
                let sig = pair
                    .sign(schnorrkel::signing_context(ctx.as_bytes()).bytes(message.as_bytes()));
                println!("{:<16}{}", "Key:", key.meta.name.green().bold());
                println!("{:<16}{}", "SS58 Address:", key.address);
                println!("{:<16}{ctx}", "Context:");
                println!("{:<16}{message}", "Message:");
                println!("{:<16}0x{}", "Signature:", hex::encode(sig.to_bytes()));
            }
            Command::Verify {
                ctx,
                message,
                signature,
                address,
            } => {
                let pk_bytes = if let Some(address) = address {
                    if let Some(encoded) = address.strip_prefix("0x") {
                        hex::decode(encoded).map_err(Into::into)
                    } else {
                        ss58::decode(address.as_bytes(), 32)
                    }
                } else {
                    let key = keyring.primary()?;
                    ss58::decode(key.address.as_bytes(), 32)
                }?;

                let pk = PublicKey::from_bytes(&pk_bytes)
                    .map_err(|e| anyhow!("Failed to decode public key, {e}"))?;

                let result = if pk
                    .verify(
                        schnorrkel::signing_context(ctx.as_bytes()).bytes(message.as_bytes()),
                        &Signature::from_bytes(&hex::decode(signature.trim_start_matches("0x"))?)
                            .map_err(|e| anyhow!("Failed to decode signature, {e}"))?,
                    )
                    .is_ok()
                {
                    "Verified".green().bold()
                } else {
                    "Not Verified".red().bold()
                };

                println!("{:<16}{result}", "Result:");
                println!("{:<16}{ctx}", "Context:");
                println!("{:<16}{message}", "Message:");
                println!("{:<16}0x{signature}", "Signature:");
                println!("{:<16}0x{}", "Public Key:", hex::encode(&pk_bytes));
                println!("{:<16}{}", "SS58 Address:", ss58::encode(&pk_bytes));
            }
        }
        Ok(())
    }

    /// Print a single key.
    fn print_key(key: &Keystore) {
        println!("Name:         {}", key.meta.name.to_string().bold());
        println!("VARA Address: {}", key.address.to_string().underline());
    }
}
