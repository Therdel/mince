// Rust doesn't directly expose __attribute__((constructor)), but this
// is how GNU implements it.
// source: https://github.com/oberien/refunct-tas/blob/9814e0e266e644613f1374dd411a16bde991282f/rtil/src/native/linux/mod.rs
#[link_section=".init_array"]
#[allow(dead_code)]
static INITIALIZE_CTOR: extern "C" fn() = initialize;

// source: https://docs.oracle.com/cd/E19120-01/open.solaris/819-0690/chapter2-48195/index.html
#[link_section=".fini_array"]
#[allow(dead_code)]
static DEINITIALIZE_DTOR: extern "C" fn() = deinitialize;

extern "C" fn initialize() {
    crate::initialize()
}

extern "C" fn deinitialize() {
    crate::deinitialize()
}

pub fn exit() {
    // todo: Do lib ref counter decrement and wait for main thread to finish
}