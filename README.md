# The SOFT Protocol

The SOFT (Simple One File Transfer) protocol has the goal of enabling robust file transfers over the network encapsulated in UDP datagrams. The protocol transports one file per connection.

## Requirements
- Rust version 1.53.0 or higher

## Build Server and Client
```
make app # on linux
```
or
```
cargo build --release
```

## Generate RFC
```
make rfc # on linux
```
