# Builds Stag gRPC server
build-grpc:
  @echo 'Building Stag gRPC server...'
  cd stag-grpc && cargo build

# Builds Stag gRPC server in release mode
build-grpc-release:
  @echo 'Building Stag gRPC server in release mode...'
  cd stag-grpc && cargo build --release

# Builds Stag UI
build-ui:
  @echo 'Building Stag UI...'
  cd stag-ui && trunk build

# Builds Stag UI in release mode
build-ui-release:
  @echo 'Building Stag UI in release mode...'
  cd stag-ui && RUSTFLAGS='-C link-arg=-s' trunk build --release

# Starts auto-reloading Stag UI server for development
serve-ui:
  @echo 'Serving Stag UI...'
  cd stag-ui && trunk serve --open

# Run unit and integration tests
test:
  @echo 'Running unit and integration tests...'
  cd stag-api && cargo test --no-default-features --features ethermint,sqlite-storage,mnemonic-signer,reqwest-client,tracing-event-handler

# Run browser tests
browser-test:
  @echo 'Running browser tests...'
  cd stag-api && wasm-pack test --chrome --headless --no-default-features --features ethermint,indexed-db-storage,mnemonic-signer,reqwest-client,tracing-event-handler

# Run grpc tests
grpc-test:
  @echo 'Running gRPC tests...'
  cd stag-grpc && cargo test --no-default-features --features ethermint,sqlite-storage,mnemonic-signer

# Run unit and integration tests (with instrumentation with lcov output)
coverage:
  @echo 'Running unit and integration test (with instrumentation with lcov output)...'
  cd stag-api && cargo +nightly llvm-cov --lcov --no-default-features --features ethermint,sqlite-storage,mnemonic-signer,reqwest-client,tracing-event-handler --output-path lcov.info

# Run unit and integration tests (with instrumentation with html output)
coverage-html:
  @echo 'Running unit and integration test (with instrumentation with html output)...'
  cd stag-api && cargo +nightly llvm-cov --html --no-default-features --features ethermint,sqlite-storage,mnemonic-signer,reqwest-client,tracing-event-handler
