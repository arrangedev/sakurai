=====================================================

 ███████╗ █████╗ ██╗  ██╗██╗   ██╗██████╗  █████╗ ██╗
 ██╔════╝██╔══██╗██║ ██╔╝██║   ██║██╔══██╗██╔══██╗██║
 ███████╗███████║█████╔╝ ██║   ██║██████╔╝███████║██║
 ╚════██║██╔══██║██╔═██╗ ██║   ██║██╔══██╗██╔══██║██║
 ███████║██║  ██║██║  ██╗╚██████╔╝██║  ██║██║  ██║██║
 ╚══════╝╚═╝  ╚═╝╚═╝  ╚═╝ ╚═════╝ ╚═╝  ╚═╝╚═╝  ╚═╝╚═╝

=====================================================
        HIGH PERFORMANCE RUST DATA STRUCTURES
=====================================================

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

Additional testing with miri (check for UB):

```bash
cargo +nightly miri test
```

## Safety

Sakurai uses `unsafe` code internally for performance, but provides safe APIs. This isn't for the weak -- use at your own risk.