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

use enclave::enclave_error::Error;

pub trait EnclaveService {
    // Calls from Engine to the Enclave

    /// Initialize Wait Certificate
    fn initialize_wait_certificate(
        prev_wait_cert: &str,
        prev_wait_cert_sig: &str,
        validator_id: &str,
        poet_pub_key: &str
    ) -> Result<u64, Error>;

    /// Finalize Wait Certificate
    fn finalize_wait_certificate(
        prev_wait_cert: &str,
        prev_wait_cert_sig: &str,
        prev_block_id: &str,
        block_summary: &str,
        wait_time: u64,
    ) -> Result<SgxWaitCertificate, Error>;

    /// Verify the wait certificate in serialized form
    fn verify_wait_certificate(
        wait_cert: &str,
        wait_cert_sign: &str,
        poet_pub_key: &str,
    ) -> Result<bool, Error>;

    /// Release the wait certificate
    fn release_wait_certificate(
        wait_cert: &str,
    ) -> Result<bool, Error>;
}