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

use bincode::ErrorKind;
use std::{error::Error, fmt};

#[derive(Debug)]
pub struct SgxStructError {
    inner: String,
}

impl std::fmt::Display for SgxStructError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "SgxStructError: {:?}", &self.inner)
    }
}

impl From<String> for SgxStructError {
    fn from(inner: String) -> SgxStructError {
        SgxStructError { inner }
    }
}

impl From<&'static str> for SgxStructError {
    fn from(inner: &'static str) -> SgxStructError {
        SgxStructError {
            inner: inner.to_string(),
        }
    }
}

impl From<Box<dyn Error>> for SgxStructError {
    fn from(err: Box<dyn Error>) -> SgxStructError {
        SgxStructError {
            inner: format!("{:?}", err),
        }
    }
}

impl From<Box<ErrorKind>> for SgxStructError {
    fn from(err: Box<ErrorKind>) -> SgxStructError {
        SgxStructError {
            inner: format!("{:?}", err),
        }
    }
}

impl Error for SgxStructError {
    fn description(&self) -> &str {
        &self.inner
    }

    fn cause(&self) -> Option<&dyn Error> {
        // Generic error, underlying cause isn't tracked
        None
    }
}
