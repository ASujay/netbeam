# netbeam ⚡

A minimal terminal tool for sending files over a local network. No internet, no cloud, no setup.

---

## Installation

Requires the [Rust toolchain](https://rustup.rs).

```bash
git clone https://github.com/ASujay/netbeam
cd netbeam
cargo build --release
```

The binary will be at `target/release/netbeam`. Move it to your PATH:

```bash
# Linux / macOS
cp target/release/netbeam /usr/local/bin/

# Windows (run as administrator)
copy target\release\netbeam.exe C:\Windows\System32\
```

---

## Usage

On the receiving machine:

```bash
netbeam receive
```

On the sending machine:

```bash
netbeam send <file_path>
```

netbeam will scan the local network, display all available receivers by name and IP, and let you pick one. The file is then transferred directly over TCP.

```
Scanning for receivers...

  1. rahul-laptop        192.168.29.143
  2. DESKTOP-XYZ123      192.168.29.2

Select a receiver: 1

Sending photo.jpg (3.2 MB)...
[====================] 100% — done
```

On the receiver side:

```
netbeam running on: rahul-laptop
Listening for incoming files...

Receiving photo.jpg (3.2 MB)...
[====================] 100% — saved to photo.jpg
```

The receiver keeps listening after each transfer until stopped with `Ctrl+C`. The sender exits once the transfer is complete. If a file with the same name already exists in the receiver's current directory, it is overwritten.

---

## Architecture

### Overview

netbeam is a single binary that operates in one of two modes depending on the subcommand. Both modes use port `22690` — TCP for file transfer and UDP for discovery.

```
SENDER                                        RECEIVER
──────                                        ────────
netbeam send photo.jpg                        netbeam receive
│                                             │
│                                             ├── Thread 1: UDP listener (background)
│                                             │     binds 0.0.0.0:22690
│                                             │     responds to discovery packets forever
│                                             │
│                                             └── Thread 2: TCP listener (background)
│                                                   binds 0.0.0.0:22690
│                                                   accepts connections forever
│                                                   spawns Thread 3 per transfer
│
├── UDP broadcast → 255.255.255.255:22690
│   "NETBEAM_DISCOVER"
│                                             Thread 1 replies:
│◀── "NETBEAM_HERE:rahul-laptop" ────────────────────┤
│
├── display list, user picks receiver
│
├── TCP connect → 192.168.29.143:22690
│                                             Thread 2 accepts, spawns Thread 3
│                                             │
├── send protocol header + file bytes         │
│                                             Thread 3 reads header + writes to disk
│                                             prints transfer complete
│                                             exits
│
└── exits
```

### Discovery — UDP Broadcast

When `netbeam send` is invoked, it enables `SO_BROADCAST` on a UDP socket and sends a `NETBEAM_DISCOVER` packet to `255.255.255.255:22690`. Every machine on the local network receives this packet. Machines running `netbeam receive` recognize it and reply with `NETBEAM_HERE:<hostname>`. The sender collects replies for 2 seconds, then displays the list.

The receiver learns the sender's IP for free from the UDP packet metadata — no extra handshake needed.

### Transfer — TCP with a framing protocol

TCP is a raw byte stream with no concept of files. netbeam defines a small header that the sender transmits before the file bytes:

```
[ 2 bytes ]  length of filename (big-endian u16)
[ N bytes ]  filename (UTF-8)
[ 8 bytes ]  file size in bytes (big-endian u64)
[ ...     ]  raw file bytes, streamed in chunks
```

The receiver reads the header first to learn the filename and exact byte count, then reads precisely that many bytes from the stream and writes them to disk in chunks. The file is never fully loaded into memory on either side.

### Concurrency model

netbeam uses OS threads — no async runtime. The receiver runs two long-lived threads from startup: one for UDP discovery replies, one for TCP connection acceptance. Each accepted TCP connection is handed off to a new short-lived thread that handles only that transfer and exits when done. The main thread blocks on stdin, keeping the process alive until `Ctrl+C`.

---

## Platform support

| Platform | Status |
|----------|--------|
| Linux    | ✅ |
| macOS    | ✅ |
| Windows  | ✅ |

---

## License

MIT