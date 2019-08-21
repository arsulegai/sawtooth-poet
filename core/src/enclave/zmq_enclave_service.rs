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
use sawtooth_sdk::messaging::stream::MessageSender;
use sawtooth_sdk::messaging::zmq_stream::ZmqMessageSender;
use enclave::enclave_service::EnclaveService;

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
            $ok => Ok(()),
            status => Err(Error::ReceiveError(format!(
                "Failed with status {:?}",
                status
            ))),
        }
    };
}

impl ZmqEnclaveService {
    fn send_to(
        &mut self,
        enclave: &EnclaveId,
        message_type: &str,
        payload: Vec<u8>,
    ) -> Result<(), Error> {
        let mut request = EngineSendToRequest::new();
        request.set_content(payload);
        request.set_message_type(message_type.into());
        request.set_receiver_id((*enclave).clone());

        let response: EngineSendToResponse = self.rpc(
            &request,
            Message_MessageType::ENGINE_SEND_TO_REQUEST,
            Message_MessageType::ENGINE_SEND_TO_RESPONSE,
        )?;

        check_ok!(response, EngineSendToResponse_Status::OK)
    }
}

impl EnclaveService for ZmqEnclaveService {
    fn initialize_wait_certificate(prev_wait_cert: &str, prev_wait_cert_sig: &str, validator_id: &str, poet_pub_key: &str) -> Result<u64, Error> {
        unimplemented!()
    }

    fn finalize_wait_certificate(prev_wait_cert: &str, prev_wait_cert_sig: &str, prev_block_id: &str, block_summary: &str, wait_time: u64) -> Result<_, Error> {
        unimplemented!()
    }

    fn verify_wait_certificate(wait_cert: &str, wait_cert_sign: &str, poet_pub_key: &str) -> Result<bool, Error> {
        unimplemented!()
    }

    fn release_wait_certificate(wait_cert: &str) -> Result<bool, Error> {
        unimplemented!()
    }
}