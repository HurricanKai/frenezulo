# Frenezulo

A WASM-embedding webserver build on top of [submillisecond](https://github.com/lunatic-solutions/submillisecond) and [lunatic](https://github.com/lunatic-solutions/lunatic).

Build to serve as an entry point for microservices compiled to WASM.
By utilizing WASM isolation it can scale services down to zero or up to infinity, depending on demand.

Note: At this time some changes to upstream repositories are required, primarily the upstream [lunatic-rs](https://github.com/lunatic-solutions/lunatic-rs) does not expose a function to spawn WASM modules with a tag & configuration, which is required.

Services can be registered locally, or added via an endpoint (/services/add)

** WARNING **
DO NOT EXPOSE THIS SERVER TO THE INTERNET DIRECTLY. THIS WILL ALLOW ARBITRARY WASM MODULES TO BE EXECUTED.
Due to the nature of WASM this does _not_ pose a security thread to the underlying system, but grants arbitrary execution permissions.

## Endpoints

Each registered service gets one endpoint under it's prefix, for example the service with the prefix `test` serves all requests to `/test/*`, including `/test/` and `/test`.

Only one prefix is reserved at this time, `services`, which is used to manage registered services.

## Performance

- Far below 1ms response times with keep-alive connections, eliminating overhead of establishing the connection
- With the overhead of establishing the connection ~0.5ms response time at full saturation.

Note that the above times are _not_ for a full TCP accept queue. If the accept queue is saturated newly enqueued requests may experience up to 500ms of delay (at the default accept queue size of 1024).
This should never happen in real-world scenarios though. Running a load balancer of some kind is critical in production scenarios.
