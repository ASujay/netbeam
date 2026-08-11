# netbeam ⚡

Netbeam is a work-in-progress terminal application for sharing files directly between devices on the same local network. The goal is an AirDrop-like workflow without an internet connection, cloud storage, or a graphical interface.

> [!IMPORTANT]
> Netbeam is not ready for file transfers yet. UDP device discovery is currently under development; the TCP transfer protocol and terminal interface are planned.

## Current status

Implemented:

- Separate `send` and `receive` modes
- IPv4 UDP broadcast discovery on port `11665`
- Discovery request, receiver information, and acknowledgement packets
- Retransmission of unacknowledged receiver information
- OS-thread-based workers with channels for application events
- Packet encoding, decoding, and malformed-packet tests

Planned:

- Display discovered devices and allow the user to select one
- Transfer files over TCP on port `11666`
- Stream files without loading them entirely into memory
- Transfer progress and terminal UI
- Graceful shutdown and complete worker error supervision
- Broader Windows, macOS, and Linux testing

## Building

Netbeam requires the [Rust toolchain](https://rustup.rs).

```bash
git clone https://github.com/ASujay/netbeam.git
cd netbeam
cargo build
```

The development binary will be written to `target/debug/netbeam`.

To create an optimized build:

```bash
cargo build --release
```

## Current usage

Start receiver mode on one device:

```bash
cargo run -- receive
```

Start sender discovery mode on another device connected to the same local network:

```bash
cargo run -- send
```

At this stage, sender mode broadcasts discovery packets periodically and receiver mode responds to them. There is no file-path argument or file transfer yet.

Depending on the operating system, you may need to allow Netbeam through the local firewall. VPNs, virtual network adapters, and networks that isolate clients can interfere with UDP broadcast discovery.

## Discovery protocol

Both modes bind an IPv4 UDP socket to `0.0.0.0:11665`.

```text
SENDER                                      RECEIVER
  |                                             |
  |----- CONN (UDP broadcast) ----------------->|
  |                                             |
  |<---- INFO { TCP port, request ID } ----------|
  |                                             |
  |----- ACKN { request ID } ------------------->|
  |                                             |
```

The packets currently use a compact binary representation:

| Packet | Layout |
| --- | --- |
| `CONN` | 4-byte `CONN` identifier |
| `INFO` | 4-byte `INFO` identifier, little-endian `u16` TCP port, little-endian `u64` request ID, UTF-8 hostname bytes |
| `ACKN` | 4-byte `ACKN` identifier, little-endian `u64` request ID |

The receiver retains unacknowledged requests and retransmits `INFO` every five seconds until it receives the corresponding `ACKN`.

## Current architecture

Netbeam uses operating-system threads rather than an async runtime.

- `app.rs` parses the mode, initializes the selected module, and owns application state.
- `sender.rs` owns the sender discovery socket and starts broadcast and reply-listener workers.
- `receiver.rs` owns the receiver discovery socket and starts reply and retransmission workers.
- `protocol.rs` implements discovery network behavior.
- `packet.rs` defines discovery packet encoding and decoding.
- `event.rs` passes worker events back to application state.
- `thread.rs` provides shared shutdown state and worker grouping.
- `registry.rs` stores discovered devices and pending requests.

## Development checks

```bash
cargo fmt -- --check
cargo check
cargo test
```

## Platform status

Cross-platform behavior is still being tested.

| Platform | Status |
| --- | --- |
| Windows | UDP discovery under development |
| macOS | UDP discovery under development |
| Linux | Not yet verified |

## License

MIT
