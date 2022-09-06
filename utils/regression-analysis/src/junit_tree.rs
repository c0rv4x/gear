// This file is part of Gear.

// Copyright (C) 2021-2022 Gear Technologies Inc.
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

use junit_common::TestSuites;
use std::{collections::BTreeMap, str::FromStr};

pub fn build_tree<Filter>(
    filter: Filter,
    test_suites: TestSuites,
) -> BTreeMap<String, BTreeMap<String, f64>>
where
    Filter: Fn(&str) -> bool,
{
    test_suites
        .testsuite
        .into_iter()
        .filter_map(|test_suite| {
            if !filter(&test_suite.name) {
                return None;
            }

            let pallet_suite = test_suite
                .testcase
                .into_iter()
                .map(|test_case| (test_case.name, f64::from_str(&test_case.time).unwrap()))
                .collect::<BTreeMap<_, _>>();

            Some((test_suite.name, pallet_suite))
        })
        .collect::<BTreeMap<_, _>>()
}
