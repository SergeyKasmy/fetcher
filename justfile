clippy-all:
    cargo +stable hack --each-feature --skip nightly clippy -- -Dwarnings

test-all:
    cargo hack --each-feature test
