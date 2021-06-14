/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2021 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::error::Error;
use std::fmt::Display;
use std::fmt;

#[derive(Debug)]
pub struct GenericError {
    msg: String
}

impl Error for GenericError {}

impl Display for GenericError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Config parsing error: {}", self.msg)
    }
}

impl GenericError {
    pub fn new(s: &str) -> Box<Self> {
        Box::new(Self{msg: String::from(s)})
    }
}
