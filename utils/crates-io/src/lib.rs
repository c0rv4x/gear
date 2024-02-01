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

//! crates-io-manager library
#![deny(missing_docs)]

mod handler;
mod manifest;
mod publisher;
mod version;

pub use self::{manifest::Manifest, publisher::Publisher, version::verify};
use anyhow::Result;
use std::process::{Command, ExitStatus};

/// Required Packages without local dependencies.
pub const SAFE_DEPENDENCIES: [&str; 12] = [
    "actor-system-error",
    "galloc",
    "gear-stack-buffer",
    "gear-core-errors",
    "gear-common-codegen",
    "gear-runtime-primitives",
    "gear-sandbox-env",
    "gear-wasm-instrument",
    "gmeta-codegen",
    "gsdk-codegen",
    "gstd-codegen",
    "gsys",
];

/// Required packages with local dependencies.
///
/// NOTE: DO NOT change the order of this array.
pub const STACKED_DEPENDENCIES: [&str; 13] = [
    "gcore",
    "gmeta",
    "gear-core",
    "gear-utils",
    "gear-common",
    "gear-sandbox-host",
    "gear-lazy-pages-common",
    "gear-lazy-pages",
    "gear-runtime-interface",
    "gear-lazy-pages-interface",
    "gear-sandbox",
    "gear-core-backend",
    "gear-core-processor",
];

/// Packages need to be published.
///
/// NOTE: DO NOT change the order of this array.
pub const PACKAGES: [&str; 7] = [
    "gring",
    "gear-wasm-builder",
    "gstd",
    "gtest",
    "gsdk",
    "gclient",
    "gcli",
];

/// Alias for packages.
pub const PACKAGE_ALIAS: [(&str, &str); 2] = [
    ("gear-core-processor", "core-processor"),
    ("gear-runtime-primitives", "runtime-primitives"),
];

/// The working version of sp-wasm-interface.
pub const GP_RUNTIME_INTERFACE_VERSION: &str = "7.0.5";

/// Check the input package
pub fn check(manifest: &str) -> Result<ExitStatus> {
    Command::new("cargo")
        .arg("check")
        .arg("--lib")
        .arg("--manifest-path")
        .arg(manifest)
        .status()
        .map_err(Into::into)
}

/// Publish the input package
pub fn publish(manifest: &str) -> Result<ExitStatus> {
    Command::new("cargo")
        .arg("publish")
        .arg("--manifest-path")
        .arg(manifest)
        .arg("--allow-dirty")
        .status()
        .map_err(Into::into)
}
