#!/bin/bash
set -e

case "${1:-native}" in
    native)
        cargo run --example demo --release
        ;;
    web|wasm)
        # Requires: cargo install wasm-bindgen-cli
        #           rustup target add wasm32-unknown-unknown
        cargo build --example demo --target wasm32-unknown-unknown --release
        wasm-bindgen target/wasm32-unknown-unknown/release/examples/demo.wasm \
            --out-dir web --target web --no-typescript
        echo "Serving at http://localhost:8080"
        python3 -m http.server -d web 8080
        ;;
    *)
        echo "Usage: ./run.sh [native|web]"
        echo ""
        echo "  native   Build and run as a native app (default)"
        echo "  web      Build for wasm and serve locally"
        echo ""
        echo "Web prerequisites:"
        echo "  cargo install wasm-bindgen-cli"
        echo "  rustup target add wasm32-unknown-unknown"
        exit 1
        ;;
esac
