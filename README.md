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

## Safety

Sakurai uses a fair bit of unsafe code internally, but the outward APIs are safe Rust. While the crate is well tested; use at your own risk.
