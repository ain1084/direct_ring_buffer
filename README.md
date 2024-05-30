# Direct Ring Buffer

[![Crates.io](https://img.shields.io/crates/v/direct_ring_buffer.svg)](https://crates.io/crates/direct_ring_buffer)
[![Documentation](https://docs.rs/direct_ring_buffer/badge.svg)](https://docs.rs/direct_ring_buffer)
[![Build Status](https://github.com/ain1084/direct_ring_buffer/workflows/Rust/badge.svg)](https://github.com/ain1084/direct_ring_buffer/actions?query=workflow%3ARust)
![Crates.io License](https://img.shields.io/crates/l/direct_ring_buffer)


This crate provides a high-performance, lock-free ring buffer for single-producer,
single-consumer scenarios. The main components of this crate are the `Producer` and
`Consumer` structures, which allow for efficient data writing and reading, respectively.

## Overview

A ring buffer is a fixed-size buffer that works as a circular queue. This implementation
uses a lock-free approach, making it suitable for real-time applications where minimal
latency is crucial. The buffer supports generic types with the `Copy` trait, ensuring
that data can be efficiently copied in and out of the buffer.

## Features

- **Single-Producer, Single-Consumer**: Designed for scenarios with a single writer
    and a single reader.
- **Lock-Free**: Utilizes atomic operations for synchronization, avoiding the need
    for mutexes or other locking mechanisms.
- **Slice-Based I/O**: Supports reading and writing multiple elements at a time
    using slices, enhancing performance for bulk operations.
- **Closure-Based Access**: Provides direct access to the buffer through closures,
    allowing for flexible and efficient data processing.
- **Generic Types with Copy Trait**: The `Copy` trait is not used for copying elements.
    It is needed to prevent the use of types implementing `Drop`, as the buffer is
    allocated uninitialized.

## Example

```rust
use direct_ring_buffer::{create_ring_buffer, Producer, Consumer};

let (mut producer, mut consumer) = create_ring_buffer::<u8>(5);

// Writing data to the buffer
producer.write(|data, _offset| {
    data[..3].copy_from_slice(&[1, 2, 3]);
    3
}, None);
producer.write(|data, _offset| {
    assert_eq!(data.len(), 2);
    data.copy_from_slice(&[4, 5]);
    data.len()
}, None);
assert_eq!(producer.available(), 0);

// Reading data from the buffer
consumer.read(|data, _offset| {
    assert_eq!(data, &[1, 2, 3, 4, 5]);
    4
}, None);
consumer.read(|data, _offset| {
    assert_eq!(data, &[5]);
    1
}, None);
assert_eq!(consumer.available(), 0);
```

In this example, a ring buffer of size 5 is created. 

1. **Writing Data (Step 1):**
  The producer writes the elements \[1, 2, 3\] into the buffer.
2. **Writing Data (Step 2):**
  Next, the producer writes the elements \[4, 5\] into the buffer.
3. **Buffer Check:**
  After these write operations, it checks that the buffer is full.
4. **Reading Data (Step 1):**
  The consumer reads the first four elements \[1, 2, 3, 4\] from the buffer.
5. **Reading Data (Step 2):**
  Then, the consumer reads the remaining element \[5\].
6. **Final Buffer Check:**
  Finally, it checks that the buffer is empty.

## Tests

The test suite is located in the [`tests/tests.rs`](https://github.com/ain1084/direct_ring_buffer/blob/main/tests/tests.rs) file and includes:
- **Method Unit Tests**: Tests for individual methods to ensure correctness.
- **Multithreaded Tests**: Tests that verify the behavior of the ring buffer in concurrent scenarios.

## Safety and Performance

This implementation ensures that data is accessed safely through `unsafe` blocks with proper checks. The use of atomic operations ensures minimal overhead for synchronization, making it suitable for high-performance applications.

- **Optimized for Bulk Operations**: Designed to handle multiple elements at once, reducing overhead for batch processing. Single-element operations may incur significant overhead.

## License

Licensed under either of
- Apache License, Version 2.0
([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
