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
