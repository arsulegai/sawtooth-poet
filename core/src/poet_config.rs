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

/// Structure to read PoET configuration from toml file
#[derive(Debug, Deserialize, Clone)]
pub struct PoetConfig {
    spid: String,
    ias_url: String,
    rest_api: String,
    ias_report_key_file: String,
    poet_client_private_key_file: String,
    is_genesis_node: Option<bool>,
    genesis_batch_path: String,
    validator_pub_key: String,
    log_dir: String,
    lib_enclave_path: String,
    lib_poet_bridge_path: String,
    ias_subscription_key: String,
}

impl PoetConfig {
    /// Getters fot the members 
    pub fn get_spid(&self) -> String {
        self.spid.clone()
    }

    pub fn get_ias_url(&self) -> String {
        self.ias_url.clone()
    }

    pub fn get_rest_api(&self) -> String {
        self.rest_api.clone()
    }

    pub fn get_ias_report_key_file(&self) -> String {
        self.ias_report_key_file.clone()
    }

    pub fn get_poet_client_private_key_file(&self) -> String {
        self.poet_client_private_key_file.clone()
    }

    pub fn is_genesis(&self) -> bool {
        self.is_genesis_node.unwrap()
    }

    pub fn get_genesis_batch_path(&self) -> String {
        self.genesis_batch_path.clone()
    }

    pub fn get_validator_pub_key(&self) -> String {
        self.validator_pub_key.clone()
    }

    pub fn set_is_genesis(&mut self, is_genesis: bool) {
        self.is_genesis_node = Some(is_genesis);
    }

    pub fn set_genesis_batch_path(&mut self, path: String) {
        self.genesis_batch_path = path;
    }

    pub fn get_log_dir(&mut self) -> String {
        return self.log_dir.clone();
    }

    pub fn get_lib_enclave_path(&self) -> String {
        self.lib_enclave_path.clone()
    }

    pub fn get_lib_poet_bridge_path(&self) -> String {
        self.lib_poet_bridge_path.clone()
    }

    pub fn get_ias_subscription_key(&self) -> String {
        self.ias_subscription_key.clone()
    }

}
