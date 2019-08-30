/*
 * Copyright 2019 Intel Corporation
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 * ------------------------------------------------------------------------------
 */

use std::error;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    SendError(String),
    ReceiveError(String),
    UnexpectedError(String),
}

impl error::Error for Error {
    fn description(&self) -> &str {
        use self::Error::*;
        match *self {
            SendError(ref s) => s,
            ReceiveError(ref s) => s,
            UnexpectedError(ref s) => s,
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        use self::Error::*;
        match *self {
            SendError(_) => None,
            ReceiveError(_) => None,
            UnexpectedError(_) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::Error::*;
        match *self {
            SendError(ref s) => write!(f, "SendError: {}", s),
            ReceiveError(ref s) => write!(f, "ReceiveError: {}", s),
            UnexpectedError(ref s) => write!(f, "UnexpectedError: {}", s),
        }
    }
}