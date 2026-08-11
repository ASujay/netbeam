const CONN_PACKET_IDENTIFIER: &[u8] = b"CONN";
const ACKN_PACKET_IDENTIFIER: &[u8] = b"ACKN";
const INFO_PACKET_IDENTIFIER: &[u8] = b"INFO";

#[derive(PartialEq, Debug)]
pub enum DiscoveryPacket {
    Conn,
    Info { port: u16, request_id: u64 },
    Ackn { request_id: u64 },
}

impl DiscoveryPacket {
    pub fn encode(&self) -> Vec<u8> {
        match self {
            DiscoveryPacket::Conn => CONN_PACKET_IDENTIFIER.to_vec(),
            DiscoveryPacket::Ackn { request_id } => {
                let mut ackn_packet = Vec::<u8>::with_capacity(12);
                ackn_packet.extend_from_slice(ACKN_PACKET_IDENTIFIER);
                ackn_packet.extend_from_slice(&request_id.to_le_bytes());
                ackn_packet
            }
            DiscoveryPacket::Info { port, request_id } => {
                let mut info_packet = Vec::<u8>::with_capacity(15);
                info_packet.extend_from_slice(INFO_PACKET_IDENTIFIER);
                info_packet.extend_from_slice(&port.to_le_bytes());
                info_packet.extend_from_slice(&request_id.to_le_bytes());
                info_packet
            }
        }
    }

    pub fn decode(buf: &[u8]) -> Option<Self> {
        let magic = buf.get(..4)?;
        match magic {
            CONN_PACKET_IDENTIFIER => Some(DiscoveryPacket::Conn),
            ACKN_PACKET_IDENTIFIER => {
                let request_id = u64::from_le_bytes(buf.get(4..12)?.try_into().ok()?);
                Some(DiscoveryPacket::Ackn { request_id })
            }
            INFO_PACKET_IDENTIFIER => {
                let port = u16::from_le_bytes(buf.get(4..6)?.try_into().ok()?);
                let request_id = u64::from_le_bytes(buf.get(6..14)?.try_into().ok()?);
                Some(DiscoveryPacket::Info {
                    port: port,
                    request_id: request_id,
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn create_info_packet_buffer(port: u16, request_id: u64) -> Vec<u8> {
        let mut buf = Vec::<u8>::new();
        buf.extend_from_slice(b"INFO");
        buf.extend_from_slice(&port.to_le_bytes());
        buf.extend_from_slice(&request_id.to_le_bytes());
        buf
    }

    fn create_ackn_packet_buffer(request_id: u64) -> Vec<u8> {
        let mut buf = Vec::<u8>::new();
        buf.extend_from_slice(b"ACKN");
        buf.extend_from_slice(&request_id.to_le_bytes());
        buf
    }

    #[test]
    fn decode_conn() {
        let buf = b"CONN";
        assert_eq!(DiscoveryPacket::decode(buf), Some(DiscoveryPacket::Conn));
    }

    #[test]
    fn decode_ackn() {
        let request_id: u64 = 0xFF;
        let mut buf = Vec::<u8>::new();
        buf.extend_from_slice(b"ACKN");
        buf.extend_from_slice(&request_id.to_le_bytes());
        assert_eq!(
            DiscoveryPacket::decode(buf.as_slice()),
            Some(DiscoveryPacket::Ackn {
                request_id: request_id
            })
        );
    }

    #[test]
    fn decode_info() {
        let buf = create_info_packet_buffer(16420, 16420);
        assert_eq!(
            DiscoveryPacket::decode(buf.as_slice()),
            Some(DiscoveryPacket::Info {
                port: 16420,
                request_id: 16420
            })
        );
    }

    #[test]
    fn encode_conn() {
        assert_eq!(DiscoveryPacket::Conn.encode(), b"CONN");
    }

    #[test]
    fn encode_ackn() {
        let buf = create_ackn_packet_buffer(0xFF);
        assert_eq!(DiscoveryPacket::Ackn { request_id: 0xFF }.encode(), buf);
    }

    #[test]
    fn encode_info() {
        let buf = create_info_packet_buffer(16420, 16420);
        assert_eq!(
            DiscoveryPacket::Info {
                port: 16420,
                request_id: 16420
            }
            .encode(),
            buf
        );
    }

    #[test]
    fn round_trip_conn() {
        let packet = DiscoveryPacket::Conn.encode();
        let decoded_packet = DiscoveryPacket::decode(&packet);
        assert_eq!(decoded_packet, Some(DiscoveryPacket::Conn));
    }

    #[test]
    fn round_trip_ackn() {
        let request_id: u64 = 0xFF;
        let packet = DiscoveryPacket::Ackn { request_id }.encode();
        let decoded_packet = DiscoveryPacket::decode(&packet);
        assert_eq!(decoded_packet, Some(DiscoveryPacket::Ackn { request_id }));
    }

    #[test]
    fn round_trip_info() {
        let packet = DiscoveryPacket::Info {
            port: 16420,
            request_id: 16420,
        }
        .encode();
        let deocded_packet = DiscoveryPacket::decode(&packet);
        assert_eq!(
            deocded_packet,
            Some(DiscoveryPacket::Info {
                port: 16420,
                request_id: 16420
            })
        );
    }

    #[test]
    fn decode_unknown() {
        assert_eq!(DiscoveryPacket::decode(b"ABCD"), None);
    }

    #[test]
    fn decode_short_packet() {
        assert_eq!(DiscoveryPacket::decode(b"AB"), None);
    }

    #[test]
    fn decode_truncated_info() {
        let mut buf = Vec::new();
        buf.extend_from_slice(b"INFO");
        buf.extend_from_slice(&16420u16.to_le_bytes());
        assert_eq!(DiscoveryPacket::decode(&buf), None);
    }
}
