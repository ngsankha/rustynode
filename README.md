# rustynode

A JavaScript runtime built on Mozilla's SpiderMonkey engine inspired from Node.js. It has an event driven and non-blocking model where the native bits are written in Rust.

Communication between native code and JS takes place only by passing messages. They are only strings now though.

**Note:** This is in very early stages and code is probably very buggy. Use at your own risk! :)

## Build

You need a nightly build of [Rust](https://www.rust-lang.org/) and then do:

    cargo build

This will do a debug build at `target/debug/rustynode`.

## Running

    ./target/debug/rustynode [filename]

A few sample files can be found in `examples` directory.
