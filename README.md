# grpc-streams
GRPC Event Streaming (i.e. lightweight in-memory Kafka replacement) server written in Rust

The ideas behind this project are:

- Create an easy to use in-memory event stream log
- It must be decently fast, that's why is based on async IO and GRPC
- It could be used as replacements for Kafka in small/non-fault-tolerant scenarios
- It can be integrated/embedded (i.e. in other Rust or Python apps)

TODO List:
- [x] A dummy service that is able to receive messages
- [ ] Client example
- [ ] An internal storage for messages
- [ ] And endpoint to consume messages in streaming fashion

...
