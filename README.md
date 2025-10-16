# Direct Ring Buffer

[![Crates.io](https://img.shields.io/crates/v/direct_ring_buffer.svg)](https://crates.io/crates/direct_ring_buffer)
[![Documentation](https://docs.rs/direct_ring_buffer/badge.svg)](https://docs.rs/direct_ring_buffer)
[![Build Status](https://github.com/ain1084/direct_ring_buffer/workflows/Rust/badge.svg)](https://github.com/ain1084/direct_ring_buffer/actions?query=workflow%3ARust)
![Crates.io License](https://img.shields.io/crates/l/direct_ring_buffer)

This crate provides a high-performance, lock-free ring buffer for single-producer, single-consumer scenarios, where efficient writing and reading of elements is crucial.

## Overview

A ring buffer is a fixed-size buffer that operates as a circular queue. This implementation uses a lock-free approach, making it suitable for real-time applications where minimal latency is important. The buffer supports efficient bulk operations through block-based reads and writes, making it ideal for scenarios with one producer and one consumer.

## Features

- **Single-Producer, Single-Consumer**: Designed for scenarios with a single writer and a single reader.  
- **Lock-Free**: Utilizes atomic operations for synchronization, avoiding the need for mutexes or other locking mechanisms.  
- **Slice-Based I/O**: Supports reading and writing multiple elements at a time using slices, enhancing performance for bulk operations.  
- **Closure-Based Access**: Provides direct access to the buffer through closures, allowing for flexible and efficient element processing.  
- **Single Element Read/Write**: Supports not only slice-based operations but also single element read and write operations.

## Type Constraints

The buffer requires the type `T` to implement the `Copy` trait because it operates on uninitialized memory. The usage of `Copy` depends on the operation:

- **Slice-based operations** like `read_slices` and `write_slices` do not require `Copy`, allowing direct memory access without copying elements.
- **Single-element operations** like `read_element` and `write_element` require `Copy` to safely handle the copying of individual elements.

## Example

```rust
use direct_ring_buffer::{create_ring_buffer, Producer, Consumer};

let (mut producer, mut consumer) = create_ring_buffer::<u8>(5);

// Write data to the buffer in slices
producer.write_slices(|data, _offset| {
    data[..3].copy_from_slice(&[1, 2, 3]);
    3
}, None);

producer.write_slices(|data, _offset| {
    data[..1].copy_from_slice(&[4]);
    1
}, None);

// Read the data
consumer.read_slices(|data, _offset| {
    assert_eq!(&data[..4], &[1, 2, 3, 4]);
    4
}, None);

// Test wrap-around by writing more data
producer.write_slices(|data, offset| {
    if offset == 0 {
        data[..1].copy_from_slice(&[6]);
        1
    } else if offset == 1 {
        data[..1].copy_from_slice(&[7]);
        1
    } else {
        panic!("Unexpected offset: {}", offset);
    }
}, None);

// Verify the wrap-around data
consumer.read_slices(|data, offset| {
    if offset == 0 {
        assert_eq!(data, &[6]);  // Read the last part of the buffer
    } else if offset == 1 {
        assert_eq!(data, &[7]);  // Read the wrapped-around part
    } else {
        panic!("Unexpected offset: {}", offset);  // Ensure only 0 or 1 is valid
    }
    data.len()
}, None);

// Write 5 more values to test wrap-around in one go
let test_data = [8, 9, 10, 11, 12];
producer.write_slices(|data, offset| {
    let write_len = data.len().min(test_data.len() - offset);
    data[..write_len].copy_from_slice(&test_data[offset..offset + write_len]);
    write_len
}, None);

// Read the newly written values to verify wrap-around
consumer.read_slices(|data, offset| {
    let read_len = data.len().min(test_data.len() - offset);
    assert_eq!(&data[..read_len], &test_data[offset..offset + read_len]);
    read_len
}, None);
```

**Note**: For single element operations using read_element or write_element, see the documentation in lib.rs.

## Safety and Performance

This implementation uses unsafe blocks with proper checks to ensure safe access to uninitialized memory. Atomic operations are utilized for synchronization, minimizing overhead and allowing for high-performance scenarios where low-latency and real-time constraints are critical.

## Optimized for Bulk Operations

This ring buffer is optimized for reading and writing multiple elements at once, reducing overhead for batch processing. While it supports single-element operations, bulk operations maximize performance and minimize overhead.

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
  at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
