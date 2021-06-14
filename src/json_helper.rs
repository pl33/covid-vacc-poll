/*
 * SPDX-License-Identifier: MPL-2.0
 *   Copyright (c) 2021 Philipp Le <philipp@philipple.de>.
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::fmt;
use std::error::Error;
use json::JsonValue;

#[derive(Debug)]
pub struct ParseError {
    msg: String
}

impl Error for ParseError {}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Config parsing error: {}", self.msg)
    }
}

impl ParseError {
    pub fn new(s: &str) -> Box<ParseError> {
        Box::new(ParseError{msg: String::from(s)})
    }
}

pub fn obj_to_str(obj: &JsonValue) -> Result<String, Box<dyn Error>> {
    match obj.as_str() {
        Some(val) => Ok(String::from(val)),
        None => return Err(ParseError::new("Could not load string from JSON"))
    }
}

pub fn obj_to_bool(obj: &JsonValue) -> Result<bool, Box<dyn Error>> {
    match obj.as_bool() {
        Some(val) => Ok(val),
        None => return Err(ParseError::new("Could not load bool from JSON"))
    }
}

pub fn obj_to_u16(obj: &JsonValue) -> Result<u16, Box<dyn Error>> {
    match obj.as_u16() {
        Some(val) => Ok(val),
        None => return Err(ParseError::new("Could not load u16 from JSON"))
    }
}

pub fn obj_to_u32(obj: &JsonValue) -> Result<u32, Box<dyn Error>> {
    match obj.as_u32() {
        Some(val) => Ok(val),
        None => return Err(ParseError::new("Could not load u32 from JSON"))
    }
}

pub fn to_str_array(obj: &JsonValue) -> Result<Vec<String>, Box<dyn Error>> {
    let mut arr: Vec<String> = Vec::new();
    for val in obj.members() {
        match val.as_str() {
            Some(v) => arr.push(String::from(v)),
            None => return Err(ParseError::new("Could not load string array from JSON"))
        }
    }
    Ok(arr)
}


