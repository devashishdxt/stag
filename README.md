# Stag

This repository implements IBC solo machine which can be used to interface with other machines & replicated ledgers
which speak IBC.

## Usage

### Prerequisites

Before building and using `stag`, you need to install the following dependencies:

1. Rust: https://rustup.rs/
1. Just: https://just.systems/
1. Protocol Buffer Compiler: https://grpc.io/docs/protoc-installation/

### Stag CLI

#### Building

To build `stag-cli`, run:

```shell
just build-cli
```

To build `stag-cli` in release mode, run:

```shell
just build-cli-release
```

#### Installing

To install `stag-cli`, run:

```shell
just install-cli
```

> For documentation on using `stag-cli`, refer [./stag-cli/README.md](./stag-cli/README.md)

### Stag gRPC Server

#### Building

To build `stag-grpc`, run:

```shell
just build-grpc
```

To build `stag-grpc` in release mode, run:

```shell
just build-grpc-release
```

#### Installing

To install `stag-grpc`, run:

```shell
just install-grpc
```

#### Running

To run `stag-grpc`, run:

```shell
stag run-grpc
```

> For more details on configuration options, run `stag-grpc start --help` after installing gRPC server on your local
machine.

### Stag UI

#### Prerequisites

1. NodeJS: https://nodejs.org/en/ (for `tailwindcss`)
1. wasm-pack: https://rustwasm.github.io/wasm-pack/ (Trunk automatically installs it but it's better to install it
   yourself)
1. Trunk: https://trunkrs.dev/

#### Building

To build `stag-ui`, run:

```shell
just build-ui
```

To build `stag-ui` in release mode, run:

```shell
just build-ui-release
```

These will put all the built artifacts in `stag-ui/dist` directory.

#### Running

To start an auto-reloading development server for `stag-ui`, run:

```shell
just serve-ui
```

## Testing

### Prerequisites

In addition to the above dependencies, you need to install the following dependencies for testing:

1. Ignite CLI: https://ignite.com/cli

### Running Tests

Before running integration tests, you need to start a local blockchain:

```shell
ignite scaffold chain github.com/devashish/mars --no-module
cp mars-config.yml ./mars/config.yml
cd mars
ignite chain serve
```

#### Integration tests (using SQLite)

In a new terminal window, run:

```shell
just test
```

To run coverage tests, you'll need nightly Rust and `llvm-tools-preview` component.

```shell
rustup toolchain install nightly
rustup component add --toolchain nightly llvm-tools-preview
```

To get coverage report (in html format), run:

```shell
just coverage-html
```

#### Browser tests (using IndexedDB)

In a new terminal window, run (this will use Google Chrome to run browser tests, so, make sure that it is installed on
your machine):

```shell
just browser-test
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as
defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
