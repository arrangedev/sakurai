```

███████╗ █████╗ ██╗  ██╗██╗   ██╗██████╗  █████╗ ██╗
██╔════╝██╔══██╗██║ ██╔╝██║   ██║██╔══██╗██╔══██╗██║
███████╗███████║█████╔╝ ██║   ██║██████╔╝███████║██║
╚════██║██╔══██║██╔═██╗ ██║   ██║██╔══██╗██╔══██║██║
███████║██║  ██║██║  ██╗╚██████╔╝██║  ██║██║  ██║██║
╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝

```

## Usage

Add to `Cargo.toml`:

```toml
[dependencies]
sakurai = "1.0.0"
```

## Testing

Run the test suite:

```bash
cargo test
```

Run UB checks:

```bash
cargo +nightly miri test
```

## Safety & Disclaimers

Sakurai uses a fair bit of unsafe code internally, but the outward APIs are safe Rust. 

While this crate is well tested, it's not recommended for production currently as some APIs are unstable (notably BTree). Use at your own risk.
