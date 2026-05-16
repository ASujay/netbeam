use std::net::Ipv4Addr;

const DISCOVERY: &'static str = "DISCOVERY";
const COMPLETE: &'static str = "COMPLETE";

#[derive(Debug)]
pub struct DiscoveryAckPacket {
    pub hostname: String,
    pub ip: Ipv4Addr,
}

#[derive(Debug)]
pub struct DataPacket<'a> {
    pub size: usize,
    pub bytes: &'a [u8],
}

/*
    discovery acknowledgement -> | type: u8 | total_size | host: size;data | u8 | u8 | u8 | u8 |
*/

#[derive(Debug)]
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

    pub fn deserialize_discovery_ack_packet(buf: &[u8], suze: usize) -> Packet<'a> {
        let sizeof_usize = size_of::<usize>();
        let mut start_index = 0;
        let total_size = usize::from_le_bytes(buf[start_index..sizeof_usize].try_into().unwrap());
        start_index += sizeof_usize;
        let hostname_size = usize::from_le_bytes(buf[start_index..(start_index + sizeof_usize)].try_into().unwrap());
        start_index += sizeof_usize;
        let hostname = String::from_utf8(buf[start_index..(hostname_size + start_index)].to_vec()).unwrap();
        start_index += hostname_size;
        let a = buf[start_index];
        let b = buf[start_index + 1];
        let c = buf[start_index + 2];
        let d = buf[start_index + 3];
        Packet::DiscoveryAck(DiscoveryAckPacket { hostname: hostname, ip: Ipv4Addr::new(a, b, c, d) })
    }

    pub fn serialize(&self) -> Vec<u8> {
        match self {
            Packet::Discovery => Vec::from(DISCOVERY),
            Packet::DiscoveryAck(data) => {
                let mut serialized_bytes = Vec::<u8>::new();
                let hostname_bytes_len = data.hostname.len();
                let total_size = size_of::<usize>() * 2 + hostname_bytes_len + size_of::<u8>() * 4;
                serialized_bytes.push(0);           // TODO(Aniket): Remove this hardcoded value
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