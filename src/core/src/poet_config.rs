/*
 * Copyright 2018 Intel Corporation
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

/// Structure to read IAS proxy server configuration from toml file
#[derive(Debug, Deserialize, Clone)]
pub struct PoetConfig {
    spid: String,
    ias_url: String,
    spid_cert_file: String,
    validator_url: String,
    ias_report_key: String,
    validator_private_key: String,
}

impl PoetConfig {
    /// Getters fot the members
    pub fn get_spid(&self) -> String {
        return self.spid.clone();
    }

    pub fn get_ias_url(&self) -> String {
        return self.ias_url.clone();
    }

    pub fn get_spid_cert_file(&self) -> String {
        return self.spid_cert_file.clone();
    }

    pub fn get_validator_url(&self) -> String {
        return self.validator_url.clone();
    }

    pub fn get_ias_report_key(&self) -> String {
        return self.ias_report_key.clone();
    }

    pub fn get_validator_private_key(&self) -> String {
        return self.validator_private_key.clone();
    }
}
