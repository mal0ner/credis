[![progress-banner](https://backend.codecrafters.io/progress/redis/0cfaaed7-6e9b-4188-830c-c789af0b57f9)](https://app.codecrafters.io/users/codecrafters-bot?r=2qF)

This is my attempt at the _codecrafters_ build your own Redis challenge. I had fun building this and learnt a lot about rust in the process!

> In this challenge, you'll build a toy Redis clone that's capable of handling
basic commands like `PING`, `SET` and `GET`. Along the way we'll learn about
event loops, the Redis protocol and more.

**Note**: See [codecrafters.io](https://codecrafters.io) to try the challenge.

# Features

1. Custom implementation of the redis RESP (Redis Serialization Protocol) meaning this 'server' can communicate directly with the official redis CLI for the features that
   have been implemented below.
2. Async client handling over tcp.
3. Custom implementation of a number of basic redis commands:
   - GET
   - SET (with timeout)
   - INFO
   - PING
4. Partial completion of the master/slave redis instance replication with protocol-compliant three-step handshake and rdb file handling.

# Running the project

1. Ensure you have `cargo (1.54)` installed locally
2. Run `./spawn_redis_server.sh` to run the Redis server.

# TODO:
- More tests
- Finish implementing data replication
