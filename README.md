# Hijekt

[![Crates.io](https://img.shields.io/crates/v/hijekt.svg)](https://crates.io/crates/hijekt)
[![Documentation](https://docs.rs/hijekt/badge.svg)](https://docs.rs/hijekt)
[![License](https://img.shields.io/crates/l/hijekt.svg)](LICENSE)

**Feature-based compile-time code injection for Rust**

Hijekt is a procedural macro that allows you to modify functions and structs at compile-time based on Cargo features. Perfect for creating optimized builds, conditional debugging, platform-specific implementations, and more.

## Features

- **Zero runtime overhead** - all modifications happen at compile-time
- **Feature-gated** - inject code only when specific features are enabled
- **Function modification** - add initialization/cleanup, swap implementations
- **Struct modification** - add/remove fields based on build configuration
- **Clean code** - keeps your source readable while generating optimized variants

## Installation

Add your features to your `Cargo.toml` like:

```toml
[dependencies]
hijekt = "0.0.2"

[features]
debug = []
cuda = []
minimal = []
profiling = []
```

## Quick Start

```rust
use hijekt::hijekt;

// Add debug logging only when the "debug" feature is enabled
#[hijekt(feat = "debug", begin = "log_start")]
fn process_data(data: &[u8]) -> usize {
    // Your main logic here
    data.len()
}

fn log_start() {
    println!("Starting data processing...");
}
```

When compiled with `--features debug`, this expands to:
```rust
#[cfg(feature = "debug")]
fn process_data(data: &[u8]) -> usize {
    log_start();
    data.len()
}

#[cfg(not(feature = "debug"))]
fn process_data(data: &[u8]) -> usize {
    data.len()
}
```

## Function Modifications

### Begin/End Injection

Add function calls at the beginning and/or end of your functions:

```rust
#[hijekt(feat = "profiling", begin = "start_timer", end = "end_timer")]
fn expensive_computation() -> u64 {
    // Your computation here
    42
}

fn start_timer() { /* profiling code */ }
fn end_timer() { /* profiling code */ }
```

### Implementation Swapping

Replace entire function bodies for different build targets:

```rust
#[hijekt(feat = "cuda", swap = "gpu_implementation")]
fn matrix_multiply(a: &[f32], b: &[f32]) -> Vec<f32> {
    // CPU fallback implementation
    cpu_multiply(a, b)
}

fn gpu_implementation() -> Vec<f32> {
    // CUDA-accelerated version
    cuda_multiply()
}
```

### Code Removal

Remove debugging code in release builds:

```rust
#[hijekt(feat = "release", rm = "debug_print")]
fn optimized_function() -> i32 {
    debug_print(); // Removed when "release" feature is active
    42
}
```

## Struct Modifications

### Remove Fields

Create minimal structs for embedded or lightweight builds:

```rust
#[hijekt(feat = "minimal", rm("debug_info", "metadata"))]
#[derive(Debug)]
struct Config {
    name: String,
    value: u32,
    debug_info: String,  // Removed in minimal builds
    metadata: Vec<u8>,   // Removed in minimal builds
}
```

### Add Fields

Extend structs with additional functionality:

```rust
#[hijekt(feat = "extended", add("cache: HashMap<String, String>", "metrics: Metrics"))]
struct Server {
    host: String,
    port: u16,
    // cache and metrics fields added when "extended" feature is enabled
}
```

### Auto-Generated Field Names

Let Hijekt generate field names from types:

```rust
#[hijekt(feat = "logging", add("String", "Vec<u8>"))]
struct Worker {
    id: u32,
    // Adds: hijekt_string: String, hijekt_vec_u8: Vec<u8>
}
```

## Complex Example

Combine multiple modifications for sophisticated feature management:

```rust
#[hijekt(
    feat = "production",
    begin = "init_metrics",
    end = "cleanup_metrics",
    rm = "debug_code",
    add("metrics: MetricsCollector", "connection_pool: Pool")
)]
struct DatabaseService {
    host: String,
    port: u16,
    debug_code: String,  // Removed in production
    // metrics and connection_pool added in production
}

fn init_metrics() { /* Initialize metrics collection */ }
fn cleanup_metrics() { /* Clean up metrics */ }
```

## Attribute Options

| Option | Description | Example |
|--------|-------------|---------|
| `feat` | Feature name to gate the modifications | `feat = "debug"` |
| `begin` | Function to call at the beginning | `begin = "init"` |
| `end` | Function to call at the end | `end = "cleanup"` |
| `swap` | Replace function body with call to this function | `swap = "optimized_impl"` |
| `rm` | Remove functions/fields (single or multiple) | `rm = "debug_fn"` or `rm("field1", "field2")` |
| `add` | Add struct fields (single or multiple) | `add = "field: Type"` or `add("field1: Type1", "Type2")` |

## Use Cases

- **Platform-specific optimizations** - CUDA/OpenCL for GPU, SIMD for CPU
- **Debug vs Release builds** - extensive logging and checks in debug mode
- **Feature tiers** - basic/standard/premium functionality levels  
- **Embedded systems** - minimal resource usage configurations
- **A/B testing** - different algorithm implementations
- **Profiling builds** - add timing and measurement code

## Performance

Hijekt operates entirely at compile-time through procedural macros. There is **zero runtime overhead** - the generated code is identical to what you would write by hand with `#[cfg()]` attributes.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

