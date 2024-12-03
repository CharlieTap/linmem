# linmem

---

linmem is small rust library that provides an api compatible with wasms linear memory abstraction. 
For each wasm memory instruction you'll find an associated function exposed through linmems api.

The library was designed to be built into a static binary with a c compatible abi so it can be consumed simply over FFI.

linmem contains api calls for all memory instructions in the WebAssembly 2.0 specification with the addition of instructions from the threads proposal.