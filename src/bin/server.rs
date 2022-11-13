use core::panic;
use std::collections::HashMap;
use std::hash::Hash;
use std::net::UdpSocket;
use std::sync::mpsc::{channel, Sender};
use std::thread::{JoinHandle,Builder};
use my_dns::dns_components::sp::start_sp;
use clap::*;
use my_dns::dns_structs::dns_message::{
    DNSMessage, DNSMessageData, DNSMessageHeaders, DNSQueryInfo, QueryType,
};
use rand::seq::IteratorRandom;

fn main() {
    let arguments = Command::new("server")
        .author("Grupo 11")
        .version("1.0.0")
        .about("A CLI tool to make DNS requests")
        .args([
            Arg::new("primary")
                .action(ArgAction::Append)
                .short('p')
                .long("primary")
                .help("Creates a primary DNS server to a domain"),
            Arg::new("secondary")
                .short('s')
                .long("secondary")
                .help("Creates a secondary DNS server to a domain"),
            Arg::new("resolver")
                .short('r')
                .long("resolver")
                .help("Creates a DNS resolver"),
            Arg::new("config_dir")
                .long("config_dir")
                .help("Directory where the config files are stored"),
        ])
        .get_matches();
    
    struct ServerThreads{
        sp: HashMap<String,(JoinHandle<()>,Sender<DNSMessage>)>,
        ss: HashMap<String,(JoinHandle<()>,Sender<DNSMessage>)>,
        sr: HashMap<String,(JoinHandle<()>,Sender<DNSMessage>)>
    }

    let mut sp_threads: HashMap<String,(JoinHandle<()>,Sender<DNSMessage>)> = HashMap::new(); 
    let mut ss_threads: HashMap<String,(JoinHandle<()>,Sender<DNSMessage>)> = HashMap::new(); 
    let mut sr_threads: HashMap<String,(JoinHandle<()>,Sender<DNSMessage>)> = HashMap::new(); 
    
    let mut server_threads = ServerThreads{sp:sp_threads,ss:ss_threads,sr:sr_threads};


    let config_dir = match arguments.get_one::<String>("config_dir"){
        Some(config_dir_arg) => config_dir_arg,
        None => {panic!("No config directory specified")}
    };
    match arguments.get_many::<String>("primary"){
        Some(domains) => {
            for domain in domains{
                let (sender,receiver) = channel::<DNSMessage>();
                let config_dir_cloned = config_dir.to_owned();
                let domain_name_cloned = domain.to_owned();
                let thread_builder = Builder::new().name(format!("SP_{}",domain));
                let thread_handle = thread_builder.spawn(move || start_sp(domain_name_cloned,config_dir_cloned,receiver)).unwrap();
                server_threads.sp.insert(domain.to_owned(),(thread_handle,sender));
                
            }
        },
        None => println!("No primary domains received")
    };

    let main_socket = match UdpSocket::bind("127.0.0.1:5454"){
        Ok(socket) => socket,
        Err(err) => panic!("{err}")
    };

    loop{
        
        
        let mut main_buffer = [0;1000];
        let (num_of_bytes,src_addr) = match main_socket.recv_from(&mut main_buffer){
            Ok(nob_sa) => nob_sa,
            Err(_) => continue
        };

        let incoming_dns_query : DNSMessage = match bincode::deserialize(&main_buffer.to_vec()){
            Ok(dns_message) => dns_message,
            Err(err) => {println!("Malformed query received");continue}
        };

        let mut rng = rand::thread_rng();

        let (_,thread_query_sender) = match server_threads.sp.get(&incoming_dns_query.data.query_info.name){
            Some(handle_and_sender) => handle_and_sender,
            None => match server_threads.ss.get(&incoming_dns_query.data.query_info.name) {
                Some(handle_and_sender) => handle_and_sender,
                None => match server_threads.sr.values().choose(&mut rng) {
                    Some(handle_and_sender) => handle_and_sender,
                    None => {println!("No component can answer your query");continue}
                }
            } 
        };
        
        match thread_query_sender.send(incoming_dns_query){
           Ok(_) => continue,
           Err(_err) => println!("Thread has closed it's receiver end of the channel")
        };
        


    }
        
}




