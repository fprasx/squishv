pack:
    wasm-pack build --target bundler riscv --out-dir ../debugger/src/pkg

build:
    just pack && npm --prefix debugger run build

dev:
    npm --prefix debugger run dev

format:
    npx rome format \
        --indent-style space \
        --indent-size 4 \
        --line-width 80 \
        --quote-style single \
        --write . \
    && cargo fmt --manifest-path riscv/Cargo.toml