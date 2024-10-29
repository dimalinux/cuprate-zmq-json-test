# cuprate-zmq-json-test

Tests that Cuprate ZMQ JSON library can deserialize live data from
monerod and serialize it back into identical JSON.

## Usage

```text
cuprate-zmq-json-test [--endpoint ENDPOINT]
```

The default endpoint is `tcp://127.0.0.1:18084` or the value of the `ZMQ_PUB`
environment variable if set.
