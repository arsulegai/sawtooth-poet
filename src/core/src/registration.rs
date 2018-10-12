/*
 Copyright 2018 Intel Corporation

 Licensed under the Apache License, Version 2.0 (the "License");
 you may not use this file except in compliance with the License.
 You may obtain a copy of the License at

     http://www.apache.org/licenses/LICENSE-2.0

 Unless required by applicable law or agreed to in writing, software
 distributed under the License is distributed on an "AS IS" BASIS,
 WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 See the License for the specific language governing permissions and
 limitations under the License.
------------------------------------------------------------------------------
*/

use openssl::sha::{sha256,
                   sha512};
use poet2_util::{read_file_as_string,
                 to_hex_string};
use protobuf::{Message,
               RepeatedField};
use sawtooth_sdk::{messages::{batch::{Batch,
                                      BatchHeader,
                                      BatchList},
                              transaction::{Transaction,
                                            TransactionHeader}},
                   signing::{create_context,
                             PrivateKey,
                             PublicKey,
                             secp256k1::Secp256k1PrivateKey,
                             Signer}};
use serde_json;
use std::str::from_utf8;
use validator_proto::{SignupInfo,
                      ValidatorRegistryPayload};
use hyper::{Body,
            Client,
            Method,
            Request,
            Uri,
            header,
            header::HeaderValue};
use ias_client::client_utils::{read_response_future,
                               read_body_as_string};
use PoetConfig;

const MAXIMUM_NONCE_SIZE: usize = 32;
const VALIDATOR_REGISTRY: &str = "validator_registry";
const CONTEXT_ALGORITHM_NAME: &str = "secp256k1";
const VALIDATOR_REGISTRY_VERSION: &str = "1.0";
const VALIDATOR_MAP: &str = "validator_map";
const REGISTER_ACTION: &str = "register";
const VALIDATOR_NAME_PREFIX: &str = "validator-";
const NAMESPACE_ADDRESS_LENGTH: usize = 6;
const MAX_SETTINGS_PARTS: usize = 4;
const SETTINGS_PART_LENGTH: usize = 16;
const CONFIGSPACE_NAMESPACE: &str = "000000";
const PUBLIC_KEY_IDENTIFIER_LENGTH: usize = 8;
const DEFAULT_VALIDATOR_PRIVATE_KEY: &str = "/etc/sawtooth/keys/validator.priv";
const BATCHES_REST_API: &str = "batches";
const APPLICATION_OCTET_STREAM: &str = "application/octet-stream";

/// Function to compose a registration request and send it to validator over REST API.
/// Accepts validator private key, signup information (AVR or the quote), block_id which can be
/// used as nonce (there's registration after every K blocks, which needs to send new nonce).
///
/// Returns response from validator REST API as string.
pub fn do_register(
    config: PoetConfig,
    block_id: &[u8],
    signup_info: SignupInfo,
) -> String {

    // Read private key from default path if it's not given as input in config
    let mut key_file = config.get_validator_private_key();
    if key_file.len() == 0 {
        key_file = DEFAULT_VALIDATOR_PRIVATE_KEY.to_string();
    }
    let read_key = read_file_as_string(key_file.as_str());

    let private_key: Box<PrivateKey> =
        Box::new(
            Secp256k1PrivateKey::from_hex(read_key.as_str())
                .expect("Invalid private key"));
    let context = create_context(CONTEXT_ALGORITHM_NAME)
        .expect("Unsupported algorithm");
    let signer = Signer::new(context.as_ref(), private_key.as_ref());
    // get signer and public key from signer in hex
    let public_key = signer.get_public_key().expect("Public key not found");
    // get public key hash -> sha256 in hex
    let public_key_hash = sha256(public_key.as_hex().as_bytes());
    // nonce from SignupInfo block id
    let nonce = from_utf8(&block_id[..MAXIMUM_NONCE_SIZE])
        .expect("Error generating nonce as string")
        .to_string();

    // Construct payload and serialize it
    let verb = REGISTER_ACTION.to_string();
    let mut name = String::new();
    name.push_str(VALIDATOR_NAME_PREFIX);
    name.push_str(&public_key.as_hex()[..PUBLIC_KEY_IDENTIFIER_LENGTH]);
    let id = public_key.as_hex();
    let raw_payload =
        ValidatorRegistryPayload::new(
            verb,
            name,
            id,
            signup_info
        );
    let payload = serde_json::to_string(&raw_payload)
        .expect("Error serializing payload to string")
        .into_bytes();

    // Namespace for the TP
    let vr_namespace = sha256(VALIDATOR_REGISTRY.as_bytes())
        [..NAMESPACE_ADDRESS_LENGTH].to_string();

    // Validator map address
    let mut vr_map_address = String::new();
    vr_map_address.push_str(vr_namespace.as_str());
    vr_map_address.push_str(sha256(VALIDATOR_MAP.as_bytes()).as_str());

    // Address to lookup this transaction
    let mut vr_entry_address = String::new();
    vr_entry_address.push_str(vr_namespace.as_str());
    vr_entry_address.push_str(public_key_hash.as_str());

    // Output address for the transaction
    let output_addresses =
        [
            vr_entry_address.clone(), vr_map_address.clone()
        ];
    let input_addresses =
        [
            vr_entry_address, vr_map_address,
            get_address_for_setting("sawtooth.poet.report_public_key_pem"),
            get_address_for_setting("sawtooth.poet.valid_enclave_measurements"),
            get_address_for_setting("sawtooth.poet.valid_enclave_basenames")
        ];

    // Create transaction header
    let transaction_header =
        create_transaction_header(&input_addresses,
                                  &output_addresses,
                                  to_hex_string(&payload),
                                  public_key,
                                  nonce,
        );

    // Create transaction
    let transaction =
        create_transaction(signer,
                           transaction_header,
                           to_hex_string(&payload),
        );

    // Create batch header, batch
    let batch = create_batch(signer, transaction);

    // Create batch list
    let batch_list = create_batch_list(batch);

    // Call the batches REST API with composed payload bytes to be sent
    submit_batchlist_to_rest_api(
        config.get_validator_url().as_str(),
        BATCHES_REST_API,
        batch_list,
    )
}

/// Function to create the ```BatchList``` object, which later is serialized and sent to REST API
/// Accepts ```Batch``` as a input parameter.
fn create_batch_list(
    batch: Batch
) -> BatchList {

    // Construct batch list
    let batches = RepeatedField::from_vec(vec![batch]);
    let mut batch_list = BatchList::new();
    batch_list.set_batches(batches);
    batch_list
}

/// Function to create the ```Batch``` object, this is then added to ```BatchList```. Accepts
/// signer object and ```Transaction``` as input parameters. Constructs ```BatchHeader``` , adds
/// signature of it to ```Batch```.
fn create_batch(
    signer: Signer,
    transaction: Transaction,
) -> Batch {

    // Construct BatchHeader
    let mut batch_header = BatchHeader::new();
    // set signer public key
    let public_key = signer.get_public_key().expect("Unable to get public key");
    let transaction_ids = vec![transaction.clone()]
        .iter()
        .map(|trans| String::from(trans.get_header_signature()))
        .collect();
    batch_header.set_transaction_ids(RepeatedField::from_vec(transaction_ids));
    batch_header.set_signer_public_key(public_key);

    // Construct Batch
    let batch_header_bytes = batch_header.write_to_bytes()
        .expect("Error converting batch header to bytes");
    let signature = signer.sign(&batch_header_bytes)
        .expect("Error signing the batch header");
    let mut batch = Batch::new();
    batch.set_header_signature(signature);
    batch.set_header(batch_header_bytes);
    batch.set_transactions(RepeatedField::from_vec(vec![transaction]));
    batch
}

/// Function to create ```Transaction``` object, accepts payload, ```TransactionHeader``` and
/// ```Signer```.
fn create_transaction(
    signer: Signer,
    transaction_header: TransactionHeader,
    payload: String,
) -> Transaction {

    // Construct a transaction, it has transaction header, signature and payload
    let transaction_header_bytes = transaction_header.write_to_bytes()
        .expect("Error converting transaction header to bytes");
    let transaction_header_signature = signer.sign(&transaction_header_bytes.to_vec())
        .expect("Error signing the transaction header");
    let mut transaction = Transaction::new();
    transaction.set_header(transaction_header_bytes.to_vec());
    transaction.set_header_signature(transaction_header_signature);
    transaction.set_payload(payload.into_bytes());
    transaction
}

/// Function to construct ```TransactionHeader``` object, accepts parameters required such as
/// input and output addresses, payload, public key of transactor, nonce to be used.
fn create_transaction_header(
    input_addresses: &[String],
    output_addresses: &[String],
    payload: String,
    public_key: Box<PublicKey>,
    nonce: String,
) -> TransactionHeader {

    // Construct transaction header
    let mut transaction_header = TransactionHeader::new();
    transaction_header.set_family_name(VALIDATOR_REGISTRY.to_string());
    transaction_header.set_family_version(VALIDATOR_REGISTRY_VERSION.to_string());
    transaction_header.set_nonce(nonce);
    transaction_header.set_payload_sha512(sha512(payload.as_bytes()));
    transaction_header.set_signer_public_key(public_key.as_hex());
    transaction_header.set_batcher_public_key(public_key.as_hex());
    transaction_header.set_inputs(RepeatedField::from_vec(input_addresses.to_vec()));
    transaction_header.set_outputs(RepeatedField::from_vec(output_addresses.to_vec()));
    transaction_header
}

/// Computes the radix address for the given setting key. Keys are broken into four parts, based
/// on the dots in the string. For example, the key `a.b.c` address is computed based on `a`,
/// `b`, `c` and the empty string. A longer key, for example `a.b.c.d.e`, is still broken into
/// four parts, but the remaining pieces are in the last part: `a`, `b`, `c` and `d.e`.
/// Each of these peices has a short hash computed (the first 16 characters of its SHA256 hash in
/// hex), and is joined into a single address, with the config namespace (`000000`) added at the
/// beginning.
/// Args:
///     setting (&str): the setting key
/// Returns:
///     String: the computed address
fn get_address_for_setting(
    setting: &str
) -> String {

    // Get parts of settings key
    let setting_parts = setting.splitn(MAX_SETTINGS_PARTS, ".");

    // If settings key has less than maximum parts, then append empty string hash
    let number_of_empty_parts_required = MAX_SETTINGS_PARTS - setting_parts.len();

    // Compute final hash to be returned
    let mut final_hash: String = String::new();

    // append 16*4 = 64 address with config state namespace
    final_hash.push_str(CONFIGSPACE_NAMESPACE);
    for setting_part in setting_parts {
        let setting_part_hash =
            sha256(setting_part)[..SETTINGS_PART_LENGTH].to_string();
        final_hash.push_str(setting_part_hash.as_str());
    }

    // for final parts, compute empty string hash
    let empty_string_hash =
        sha256([])[..SETTINGS_PART_LENGTH]
            .to_string()
            .repeat(number_of_empty_parts_required);
    final_hash.push_str(empty_string_hash.as_str());

    // Return the final computed hash for the settings key
    final_hash
}

/// Sends the BatchList to the REST API
pub fn submit_batchlist_to_rest_api(
    url: &str,
    api: &str,
    batch_list: BatchList
) -> String {

    // Create request body, which in this case is batch list
    let raw_bytes = batch_list.write_to_bytes().expect("Unable to write batch list as bytes");
    let body_length = raw_bytes.len();
    let bytes = Body::from(raw_bytes.to_vec());

    // API to call
    let uri = Uri::builder()
        .path_and_query(url)
        .path_and_query(api)
        .build()
        .expect("Error constructing the REST API URI");

    // Construct client to send request
    let client = Client::new();

    // Compose POST request, to register
    let mut request = Request::new(bytes);
    *request.method_mut() = Method::POST;
    *request.uri_mut() = uri;
    request.headers_mut().insert(
        header::CONTENT_TYPE,
        HeaderValue::from_static(APPLICATION_OCTET_STREAM),
    );
    request.headers_mut().insert(
        header::CONTENT_LENGTH,
        HeaderValue::from(body_length),
    );

    // Get response as string and return
    let response_future = client.request(request);
    let read_response = read_response_future(response_future)
        .expect("Error reading response");
    let response_body = read_body_as_string(read_response.body)
        .expect("Error occurred during registration");
    response_body
}

#[cfg(test)]
mod tests {
    use sawtooth_sdk::signing::{Context,
                                secp256k1::{Secp256k1Context,
                                            Secp256k1PrivateKey}};
    use super::*;

    #[test]
    fn test_get_address_for_setting() {
        let precomputed_address = "000000ca978112ca1bbdca3e23e8160039594a2e7d2c03a9507ae2e3b0c44298fc1c14";
        let address_calculated = get_address_for_setting("a.b.c");
        assert_eq!(precomputed_address, address_calculated);

        let precomputed_address = "000000ca978112ca1bbdca3e23e8160039594a2e7d2c03a9507ae2e67adc8234459dc2";
        let address_calculated = get_address_for_setting("a.b.c.d.e");
        assert_eq!(precomputed_address, address_calculated);
    }

    #[test]
    fn test_get_transaction() {
        let dummy_transaction_bytes = "dummy bytes".as_bytes();
        let dummy_payload = "dummy payload";
        let dummy_signature = "dummy_signature";
        let transaction = create_transaction(dummy_transaction_bytes, dummy_signature.to_string(),
                                             dummy_payload.to_string());
        assert_eq!(transaction.get_header(), dummy_transaction_bytes);
        assert_eq!(transaction.get_header_signature(), dummy_signature);
        assert_eq!(transaction.get_payload(), dummy_payload.to_string().as_bytes());
    }

    #[test]
    fn test_get_transaction_header() {
        let dummy_input_addresses = ["dummy input addresses".to_string()];
        let dummy_output_addresses = ["dummy output addresses".to_string()];
        let dummy_payload = "dummy payload".to_string();
        let dummy_nonce = "dummy nonce".to_string();
        let context = Secp256k1Context::new();
        let dummy_private_key: Box<PrivateKey> = context.new_random_private_key().unwrap();
        let signer = Signer::new(&context, dummy_private_key.as_ref());
        let dummy_public_key = signer.get_public_key().unwrap();
        let transaction_header = create_transaction_header(&dummy_input_addresses,
                                                           &dummy_output_addresses, dummy_payload.clone(),
                                                           dummy_public_key, dummy_nonce.clone());
        assert_eq!(transaction_header.get_nonce(), dummy_nonce);
        assert_eq!(transaction_header.get_payload_sha512(), sha512_from_str(dummy_payload.as_str()));
        assert_eq!(transaction_header.get_nonce(), dummy_nonce);
    }

    #[test]
    fn get_test_batch() {
        let dummy_transaction_bytes = "dummy bytes".as_bytes();
        let dummy_payload = "dummy payload";
        let dummy_signature = "dummy_signature";
        let transaction = create_transaction(dummy_transaction_bytes, dummy_signature.to_string(),
                                             dummy_payload.to_string());
        let context = Secp256k1Context::new();
        let dummy_private_key: Box<PrivateKey> = context.new_random_private_key().unwrap();
        let signer1 = Signer::new(&context, dummy_private_key.as_ref());
        let signer2 = Signer::new(&context, dummy_private_key.as_ref());
        let batch = create_batch(signer1, transaction.clone());
        let dummy_public_key = signer2.get_public_key().unwrap();
        let transaction_ids = vec![transaction.clone()]
            .iter()
            .map(|trans| String::from(trans.get_header_signature()))
            .collect();
        let mut batch_header = BatchHeader::new();
        batch_header.set_transaction_ids(RepeatedField::from_vec(transaction_ids));
        batch_header.set_signer_public_key(dummy_public_key.as_hex());
        let batch_header_bytes = batch_header.write_to_bytes().unwrap();
        let signature = signer2.sign(&batch_header_bytes).unwrap();
        assert_eq!(batch.get_header_signature(), signature);
    }

    #[test]
    fn test_get_batch_list() {
        let dummy_transaction_bytes = "dummy bytes".as_bytes();
        let dummy_payload = "dummy payload";
        let dummy_signature = "dummy_signature";
        let transaction = create_transaction(dummy_transaction_bytes, dummy_signature.to_string(),
                                             dummy_payload.to_string());
        let context = Secp256k1Context::new();
        let dummy_private_key: Box<PrivateKey> = context.new_random_private_key().unwrap();
        let signer1 = Signer::new(&context, dummy_private_key.as_ref());
        let signer2 = Signer::new(&context, dummy_private_key.as_ref());
        let batch1 = create_batch(signer1, transaction.clone());
        let batch2 = create_batch(signer2, transaction.clone());
        let batch_list = create_batch_list(batch1);
        assert_eq!(batch_list.get_batches(), [batch2])
    }
}