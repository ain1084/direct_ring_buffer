# Direct Ring Buffer

[![Crates.io](https://img.shields.io/crates/v/direct_ring_buffer.svg)](https://crates.io/crates/direct_ring_buffer)
[![Documentation](https://docs.rs/direct_ring_buffer/badge.svg)](https://docs.rs/direct_ring_buffer)
[![Build Status](https://github.com/ain1084/direct_ring_buffer/workflows/Rust/badge.svg)](https://github.com/ain1084/direct_ring_buffer/actions?query=workflow%3ARust)
![Crates.io License](https://img.shields.io/crates/l/direct_ring_buffer)

This crate provides a high-performance, lock-free ring buffer for single-producer,
single-consumer scenarios. The main components of this crate are the `Producer` and
`Consumer` structures, which allow for efficient elements writing and reading,
respectively.

## Overview

A ring buffer is a fixed-size buffer that works as a circular queue. This implementation
uses a lock-free approach, making it suitable for real-time applications where minimal
latency is crucial. The buffer supports generic types with the `Copy` trait. The `Copy`
trait is mainly used to prevent the use of types implementing `Drop`, as the buffer is
allocated uninitialized.

## Features

- **Single-Producer, Single-Consumer**: Designed for scenarios with a single writer
  and a single reader.
- **Lock-Free**: Utilizes atomic operations for synchronization, avoiding the need for
  mutexes or other locking mechanisms.
- **Slice-Based I/O**: Supports reading and writing multiple elements at a time using
  slices, enhancing performance for bulk operations.
- **Closure-Based Access**: Provides direct access to the buffer through closures,
  allowing for flexible and efficient elements processing.
- **Single Element Read/Write**: Supports not only bulk operations but also single
  element read and write operations.

## Example

```rust
use direct_ring_buffer::{create_ring_buffer, Producer, Consumer};

let (mut producer, mut consumer) = create_ring_buffer::<u8>(5);

// Writing data to the buffer
producer.write_slices(|data, _offset| {
    data[..3].copy_from_slice(&[1, 2, 3]);
    3
}, None);
assert!(producer.write_element(4));
assert!(producer.write_element(5));
assert_eq!(producer.available(), 0);

// Reading data from the buffer
consumer.read_slices(|data, _offset| {
    assert_eq!(data, &[1, 2, 3, 4, 5]);
    4
}, None);
assert_eq!(consumer.read_element(), Some(5));
assert_eq!(consumer.read_element(), None);
assert_eq!(consumer.available(), 0);
```

1. **Writing Elements (Step 1):**
   The producer writes the elements \[1, 2, 3\] into the buffer using `write_slices`.
2. **Writing Elements (Step 2):**
   Next, the producer writes the element \[4\] into the buffer using `write_element`.
3. **Writing Elements (Step 3):**
   Then, the producer writes the element \[5\] into the buffer using `write_element`, filling the buffer.
4. **Buffer Check:**
   After these write operations, it checks that the buffer is full.
5. **Reading Elements (Step 1):**
   The consumer reads the first four elements \[1, 2, 3, 4\] from the buffer using `read_slices`.
6. **Reading Elements (Step 2):**
   Then, the consumer reads the remaining element \[5\] using `read_element`.
7. **Final Buffer Check:**
   Finally, it checks that the buffer is empty.

## Safety and Performance

This implementation ensures that elements are accessed safely through `unsafe` blocks
with proper checks. The use of atomic operations ensures minimal overhead for
synchronization, making it suitable for high-performance applications.

## Optimized for Bulk Operations

Designed to handle multiple elements at once, reducing overhead for batch processing.
Single-element operations may incur significant overhead.

## License

Licensed under either of
- Apache License, Version 2.0
([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)
at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
