use std::net::Ipv4Addr;

const DISCOVERY: &'static str = "DISCOVERY";
const COMPLETE: &'static str = "COMPLETE";

pub struct DiscoveryAckPacket {
    hostname: String,
    ip: Ipv4Addr,
}

pub struct DataPacket<'a> {
    size: usize,
    bytes: &'a [u8],
}

/*
    discovery acknowledgement -> | total_size | host: size;data | u8 | u8 | u8 | u8 |
*/

pub enum Packet<'a> {
    Discovery,
    DiscoveryAck(DiscoveryAckPacket),
    StandardData(DataPacket<'a>),
    Complete,
}

impl<'a> Packet<'a> {
    pub fn create_discovery_packet() -> Packet<'a> {
        Packet::Discovery
    }

    pub fn create_discovery_ack_packet(hostname: String, ip: Ipv4Addr) -> Packet<'a> {
        Packet::DiscoveryAck(DiscoveryAckPacket{hostname, ip})
    }

    pub fn create_completion_packet() -> Packet<'a> {
        return Packet::Complete
    }

    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Packet::Discovery => Vec::from(DISCOVERY),
            Packet::DiscoveryAck(data) => {
                let mut serialized_bytes = Vec::<u8>::new();
                let hostname_bytes_len = data.hostname.len();
                let total_size = size_of::<usize>() * 2 + hostname_bytes_len + size_of::<u8>() * 4; 
                serialized_bytes.extend_from_slice(&total_size.to_le_bytes());
                serialized_bytes.extend_from_slice(&hostname_bytes_len.to_le_bytes());
                serialized_bytes.extend_from_slice(&data.hostname.as_bytes());
                serialized_bytes.extend_from_slice(&data.ip.octets());
                serialized_bytes
            },
            Packet::StandardData(data) => {
                let serialized_bytes = Vec::<u8>::new();

                serialized_bytes
            },
            Packet::Complete => Vec::from(COMPLETE),
        }
    }
}