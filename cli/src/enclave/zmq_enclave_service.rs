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
    SignUpInfo,
    EnclaveInitializeRequest,
    EnclaveInitializeResponse,
    EnclaveSetSigRlRequest,
    EnclaveSetSigRlResponse,
    EnclaveCreateSignUpInfoRequest,
    EnclaveCreateSignUpInfoResponse,
    EnclaveInitializeResponse_Status,
    EnclaveSetSigRlResponse_Status,
    EnclaveCreateSignUpInfoResponse_Status,
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
    fn init_enclave(&mut self) -> Result<(), Error> {
        let mut request = EnclaveInitializeRequest::new();

        let response: EnclaveInitializeResponse = match self.rpc(
            &request,
            Message_MessageType::ENCLAVE_INITIALIZE_REQUEST,
            Message_MessageType::ENCLAVE_INITIALIZE_RESPONSE,
        ) {
            Ok(res) => res,
            Err(err) => return Err(err),
        };

        match check_ok!(response, EnclaveInitializeResponse_Status::OK) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    fn set_sig_revocation_list(
        &mut self,
        sig_rev_list: &str
    ) -> Result<(), Error> {
        let mut request = EnclaveSetSigRlRequest::new();
        request.set_signrl(sig_rev_list.to_string());

        let response: EnclaveSetSigRlResponse = match self.rpc(
            &request,
            Message_MessageType::ENCLAVE_SET_SIG_RL_REQUEST,
            Message_MessageType::ENCLAVE_SET_SIG_RL_RESPONSE,
        ) {
            Ok(res) => res,
            Err(err) => return Err(err),
        };

        match check_ok!(response, EnclaveSetSigRlResponse_Status::OK) {
            Ok(_) => Ok(()),
            Err(err) => Err(err),
        }
    }

    fn create_signup_info(&mut self) -> Result<SignUpInfo, Error> {
        let mut request = EnclaveCreateSignUpInfoRequest::new();

        let response: EnclaveCreateSignUpInfoResponse = match self.rpc(
            &request,
            Message_MessageType::ENCLAVE_CREATE_SIGNUPINFO_REQUEST,
            Message_MessageType::ENCLAVE_CREATE_SIGNUPINFO_RESPONSE,
        ) {
            Ok(res) => res,
            Err(err) => return Err(err),
        };

        match check_ok!(response, EnclaveCreateSignUpInfoResponse_Status::OK) {
            Ok(res) => Ok(res.get_signup_info()),
            Err(err) => Err(err),
        }
    }
}