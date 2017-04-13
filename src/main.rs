extern crate trust_dns as dns;

use std::net::IpAddr;

fn getaddrinfo (host: &str) -> Vec<IpAddr> {
    //TODO "" -> 127.0.0.1, ::1
    //TODO "x.x.x.x" -> x.x.x.x
    //TODO "x:...:x" -> x:...:x
    //TODO parse /etc/hosts (man 5 hosts)

    let mut resolved = Vec::new();

    match ResolvConf::get() {
        Some(conf) => {
            for addr in conf.nameservers {
                use dns::client::{Client, SyncClient};
                use dns::udp::UdpClientConnection;
                use dns::rr::domain::Name;
                use dns::rr::dns_class::DNSClass;
                use dns::rr::record_type::RecordType;
                use dns::rr::record_data::RData;
                use std::net::{IpAddr, Ipv4Addr, SocketAddr};

                let addr = SocketAddr::new(addr, 53);
                let conn = UdpClientConnection::new(addr).expect("udp_conn.new");
                let client = SyncClient::new(conn);
                let name = Name::parse(host, None).expect("name.parse");

                for answer in client.query(&name, DNSClass::IN, RecordType::A).expect("client.query").answers() {
                    //println!("{:?}", answer.rdata());
                    if let &RData::A(v4addr) = answer.rdata() {
                        resolved.push(IpAddr::V4(v4addr));
                    }
                }
                for answer in client.query(&name, DNSClass::IN, RecordType::AAAA).expect("client.query").answers() {
                    //println!("{:?}", answer.rdata());
                    if let &RData::A(v6addr) = answer.rdata() {
                        resolved.push(IpAddr::V4(v6addr));
                    }
                }
            }
        }
        None => {}
    }

    resolved
}

struct ResolvConf {
    nameservers: Vec<IpAddr>
}

impl ResolvConf {
    fn get () -> Option<ResolvConf> {
        use std::fs::File;
        use std::io::BufRead;
        use std::io::BufReader;

        let mut nameservers = Vec::new();

        match File::open("/etc/resolv.conf") {
            Ok(file) => {
                for line in BufReader::new(file).lines() {
                    match line {
                        Ok(line) => {
                            match line.split("#").next() {
                                Some(line) => {
                                    let words = line.split_whitespace().collect::<Vec<&str>>();
                                    //println!("{:?}", words);
                                    if words.len() == 2 && words[0] == "nameserver" {
                                        use std::str::FromStr;
                                        match IpAddr::from_str(words[1]) {
                                            Ok(ip) => nameservers.push(ip),
                                            Err(_e) => {}
                                        }
                                    }
                                    //TODO parse other keywords
                                }
                                None => {}
                            }
                        }
                        Err(_e) => { /*TODO*/ break; }
                    }
                }

                //println!("ns: {:?}", nameservers);

                if nameservers.is_empty() {
                    None
                } else {
                    Some(ResolvConf{ nameservers: nameservers })
                }
            }
            Err(_e) => None
        }
    }
}

fn main() {
    use std::env::args;
    println!("{:?}", getaddrinfo(&args().nth(1).expect("no host name specified")));
}
