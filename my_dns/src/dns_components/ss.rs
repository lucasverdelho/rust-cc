use std::{
    collections::HashMap,
    hash::Hash,
    ops::Add,
    path::{self, Path},
    sync::mpsc::Receiver,
};

use crate::{
    dns_parse::server_config_parse,
    dns_structs::{
        dns_message::DNSMessage, domain_database_struct::DomainDatabase,
        server_config::ServerConfig,
    },
};
use queues::*;

pub fn start_ss(domain_name: String, config: ServerConfig, receiver: Receiver<DNSMessage>) {
    let config: ServerConfig;

    match Path::new(&config_dir)
        .join(domain_name.clone().replace(".", "-").add(".conf"))
        .to_str()
    {
        Some(path) => match server_config_parse::get(path.to_string()) {
            Ok(config_parsed) => config = config_parsed,
            Err(err) => panic!("{err}"),
        },
        None => {
            panic!("no config file found for the domain_name {}", domain_name)
        }
    };

    loop {
        let dns_message = match receiver.recv() {
            Err(err) => panic!("{err}"),
            Ok(ok) => ok,
        };
        println!("SS received query of {}", dns_message.data.query_info.name);
    }
}
