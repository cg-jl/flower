# flower


A simulation of wave-like patterns with different speeds that make for a
beautiful rotating and expanding flower.

Done with WASM and Rust, with the support of `wasm-bindgen`, `cargo-generate`
and all the tools from the RustWASM Team. Much appreciated!


The simulation first calculates all the colors it will use and makes all the
canvases it needs to make up for all the images it will blit. The JavaScript is
responsible for requesting animation frames, calling the Rust code to
update the state and the color indices and updating the canvas screen so that we
can see the simulation. The Rust code is responsible for all computations, like
color computation, which image corresponds to each ball, and updating the
positions every frame, making sure they don't go off.
