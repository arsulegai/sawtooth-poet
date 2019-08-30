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

extern crate bincode;
#[macro_use]
extern crate clap;
extern crate crypto;
extern crate hyper;
extern crate ias_client;
extern crate lazy_static;
extern crate sawtooth_sdk;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate num;
extern crate openssl;
extern crate protobuf;
extern crate rand;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate hex;
extern crate serde_json;
extern crate zmq;
extern crate common;

use engine::Poet2Engine;
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;
use poet2_util::read_file_as_string;
use poet_config::PoetConfig;
use sawtooth_sdk::consensus::zmq_driver::ZmqDriver;
use std::path::Path;
use std::process;
use toml as toml_converter;
use clap::{App, Arg, SubCommand};
use registration::create_registration;
use registration::do_registration;

mod protos;
mod registration;
mod enclave;
mod cli_error;
mod enclave_wrapper;

/*
 *
 * This is the main() method.
 *
 * This is where we parse the command-line args and
 * setup important parameters like:
 * - endpoint url of the enclave process
 * - verbosity of logging
 * - generate the registration batch file or send the registration command
 *
 * @params None
 *
 */

fn main() {
    let enclave_modules = vec!["simulator", "sgx"];
    let matches = App::new("Sawtooth PoET CLI")
        .version(crate_version!())
        .about("PoET Consensus Engine CLI")
        .arg(
            Arg::with_name("verbose")
                .long("verbose")
                .short("v")
                .multiple(true)
                .required(false)
                .help("Increase the verbosity")
        )
        .arg(
            Arg::with_name("enclave-module")
                .long("enclave-module")
                .short("e")
                .takes_value(true)
                .required(true)
                .help("The enclave module to connect to")
        )
        .subcommand(
            SubCommand::with_name("registration")
                .help("Provides a subcommand for creating PoET2 registration")
                .subcommand(
                    SubCommand::with_name("create")
                        .help("Creates a batch to enroll a validator in the validator registry")
                )
                .arg(
                    Arg::with_name("enclave-module")
                        .long("enclave-module")
                        .help("identify the enclave module to query")
                        .default_value("simulator")
                        .possible_values(&enclave_modules)
                )
                .arg(
                    Arg::with_name("key")
                        .long("key")
                        .short("k")
                        .takes_value(true)
                        .default_value("/etc/sawtooth/validator.priv")
                        .help("identify file containing transaction signing key")
                )
                .arg(
                    Arg::with_name("output")
                        .default_value("poet-genesis.batch")
                        .long("output")
                        .short("o")
                        .takes_value(true)
                        .help("change default output file name for resulting batches")
                )
                .arg(
                    Arg::with_name("url")
                        .long("url")
                        .short("u")
                        .takes_value(true)
                        .required(false)
                        .help("Sends the generated batches to the URL")
                )
        )
        .subcommand(
            SubCommand::with_name("enclave")
                .help("Generates enclave setup information")
                .arg(
                    Arg::with_name("enclave-module")
                        .long("enclave-module")
                        .help("identify the enclave module to query")
                        .default_value("simulator")
                        .possible_values(&enclave_modules)
                )
                .subcommand(
                    SubCommand::with_name("measurement")
                        .help("enclave characteristic to retrieve")
                )
                .subcommand(
                    SubCommand::with_name("basename")
                        .help("enclave characteristic to retrieve")
                )
        )
        .get_matches();

    let endpoint = matches
        .value_of("url")
        .unwrap_or("tcp://localhost:8008");
    // URL is for the REST API where registration request is to be sent

    let log_level;
    match matches.occurrences_of("verbose") {
        0 => log_level = LevelFilter::Warn,
        1 => log_level = LevelFilter::Info,
        2 => log_level = LevelFilter::Debug,
        3 | _ => log_level = LevelFilter::Trace,
    }

    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d:22.22} {h({l:5.5})} | {({M}:{L}):30.30} | {m}{n}",
        )))
        .build();

    let fileout = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d:22.22} {h({l:5.5})} | {({M}:{L}):30.30} | {m}{n}",
        )))
        .build(
            Path::new(&config.get_log_dir())
                .join("poet-consensus.log")
                .to_str()
                .expect("Failed to get log file path"),
        )
        .expect("Could not build file appender");

    let log_config: Config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("fileout", Box::new(fileout)))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("fileout")
                .build(log_level),
        )
        .unwrap_or_else(|err| {
            error!("{}", err);
            process::exit(1);
        });

    log4rs::init_config(log_config).unwrap_or_else(|err| {
        error!("{}", err);
        process::exit(1);
    });

    // This is a must arg, there needs to be an enclave for engine or the CLO
    let enclave = matches.value_of("enclave-module").expect("No enclave module configured");

    info!("Connecting to the enclave service...");
    // TODO: Trying to connect to the enclave

    if let Some(matches) = matches.subcommand_matches("registration") {
        do_registration(matches, enclave).expect("Unable to register")
    } else if let Some(matches) = matches.subcommand_matches("enclave") {
        do_enclave(matches, enclave).expect("Unable to load the enclave")
    } else {
        panic!("Unexpected input");
    }
}
