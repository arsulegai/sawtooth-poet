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

use std::time::Duration;

use protobuf;
use rand::Rng;

use enclave::enclave_error::Error;
use common::messaging::stream::MessageSender;
use common::messaging::zmq_stream::ZmqMessageSender;
use enclave::enclave_service::EnclaveService;
use protos::enclave::Message_MessageType;
use protos::enclave::{
    SgxWaitCertificate,
    EnclaveInitializeWaitCertificateRequest,
    EnclaveFinalizeWaitCertificateRequest,
    EnclaveVerifyWaitCertificateRequest,
    EnclaveReleaseWaitCertificateRequest,
    EnclaveInitializeWaitCertificateResponse,
    EnclaveFinalizeWaitCertificateResponse,
    EnclaveVerifyWaitCertificateResponse,
    EnclaveReleaseWaitCertificateResponse,
    EnclaveInitializeWaitCertificateResponse_Status,
    EnclaveFinalizeWaitCertificateResponse_Status,
    EnclaveVerifyWaitCertificateResponse_Status,
    EnclaveReleaseWaitCertificateResponse_Status,
};

fn generate_correlation_id() -> String {
    const LENGTH: usize = 16;
    rand::thread_rng().gen_ascii_chars().take(LENGTH).collect()
}

pub struct ZmqEnclaveService {
    sender: ZmqMessageSender,
    timeout: Duration,
}

impl ZmqEnclaveService {
    pub fn new(sender: ZmqMessageSender, timeout: Duration) -> Self {
        ZmqEnclaveService {
            sender,
            timeout,
        }
    }

    /// Serialize and send a request, wait for the default timeout, and receive and parse an
    /// expected response.
    pub fn rpc<I: protobuf::Message, O: protobuf::Message>(
        &mut self,
        request: &I,
        request_type: Message_MessageType,
        response_type: Message_MessageType,
    ) -> Result<O, Error> {
        let corr_id = generate_correlation_id();
        let mut future = self
            .sender
            .send(request_type, &corr_id, &request.write_to_bytes()?)?;

        let msg = future.get_timeout(self.timeout)?;
        let msg_type = msg.get_message_type();
        if msg_type == response_type {
            let response = protobuf::parse_from_bytes(msg.get_content())?;
            Ok(response)
        } else {
            Err(Error::ReceiveError(format!(
                "Received unexpected message type: {:?}",
                msg_type
            )))
        }
    }
}

/// Return Ok(()) if $r.get_status() matches $ok
macro_rules! check_ok {
    ($r:expr, $ok:pat) => {
        match $r.get_status() {
            $ok => Ok($r),
            status => Err(Error::ReceiveError(format!(
                "Failed with status {:?}",
                status
            ))),
        }
    };
}

impl EnclaveService for ZmqEnclaveService {
    fn initialize_wait_certificate(
        &mut self,
        prev_wait_cert: &str,
        prev_wait_cert_sig: &str,
        validator_id: &str,
        poet_pub_key: &str
    ) -> Result<u64, Error> {
        let mut request = EnclaveInitializeWaitCertificateRequest::new();
        request.set_prev_wait_cert(prev_wait_cert.to_string());
        request.set_prev_wait_cert_sig(prev_wait_cert_sig.to_string());
        request.set_validator_id(validator_id.to_string());
        request.set_poet_pub_key(poet_pub_key.to_string());

        let response: EnclaveInitializeWaitCertificateResponse = match self.rpc(
            &request,
            Message_MessageType::ENCLAVE_INITIALIZE_WAIT_CERTIFICATE_REQUEST,
            Message_MessageType::ENCLAVE_INITIALIZE_WAIT_CERTIFICATE_RESPONSE,
        ) {
            Ok(res) => res,
            Err(err) => return Err(err),
        };

        match check_ok!(response, EnclaveInitializeWaitCertificateResponse_Status::OK) {
            Ok(res) => Ok(res.get_duration()),
            Err(err) => Err(err),
        }
    }

    fn finalize_wait_certificate(
        &mut self,
        prev_wait_cert: &str,
        prev_wait_cert_sig: &str,
        prev_block_id: &str,
        block_summary: &str,
        wait_time: u64
    ) -> Result<SgxWaitCertificate, Error> {
        let mut request = EnclaveFinalizeWaitCertificateRequest::new();
        request.set_prev_wait_cert(prev_wait_cert.to_string());
        request.set_prev_wait_cert_sig(prev_wait_cert_sig.to_string());
        request.set_prev_block_id(prev_block_id.to_string());
        request.set_block_summary(block_summary.to_string());
        request.set_wait_time(wait_time);

        let response: EnclaveFinalizeWaitCertificateResponse = match self.rpc(
            &request,
            Message_MessageType::ENCLAVE_FINALIZE_WAIT_CERTIFICATE_REQUEST,
            Message_MessageType::ENCLAVE_FINALIZE_WAIT_CERTIFICATE_RESPONSE,
        ) {
            Ok(res) => res,
            Err(err) => return Err(err),
        };

        match check_ok!(response, EnclaveFinalizeWaitCertificateResponse_Status::OK) {
            Ok(res) => Ok(res.get_sgx_wait_certificate()),
            Err(err) => Err(err),
        }
    }

    fn verify_wait_certificate(
        &mut self,
        wait_cert: &str,
        wait_cert_sig: &str,
        poet_pub_key: &str
    ) -> Result<bool, Error> {
        let mut request = EnclaveVerifyWaitCertificateRequest::new();
        request.set_wait_cert(wait_cert.to_string());
        request.set_wait_cert_sig(wait_cert_sig.to_string());
        request.set_poet_pub_key(poet_pub_key.to_string());

        let response: EnclaveVerifyWaitCertificateResponse = match self.rpc(
            &request,
            Message_MessageType::ENCLAVE_VERIFY_WAIT_CERTIFICATE_REQUEST,
            Message_MessageType::ENCLAVE_VERIFY_WAIT_CERTIFICATE_RESPONSE,
        ) {
            Ok(res) => res,
            Err(err) => return Err(err),
        };

        match check_ok!(response, EnclaveVerifyWaitCertificateResponse_Status::OK) {
            Ok(res) => Ok(res.get_status()),
            Err(err) => Err(err),
        }
    }

    fn release_wait_certificate(
        &mut self,
        wait_cert: &str
    ) -> Result<bool, Error> {
        let mut request = EnclaveReleaseWaitCertificateRequest::new();
        request.set_wait_cert(wait_cert.to_string());

        let response: EnclaveReleaseWaitCertificateResponse = match self.rpc(
            &request,
            Message_MessageType::ENCLAVE_RELEASE_WAIT_CERTIFICATE_REQUEST,
            Message_MessageType::ENCLAVE_RELEASE_WAIT_CERTIFICATE_RESPONSE,
        ) {
            Ok(res) => res,
            Err(err) => return Err(err),
        };

        match check_ok!(response, EnclaveReleaseWaitCertificateResponse_Status::OK) {
            Ok(res) => Ok(get_status()),
            Err(err) => Err(err),
        }
    }
}