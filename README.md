# Direct Ring Buffer

[![Crates.io](https://img.shields.io/crates/v/direct_ring_buffer.svg)](https://crates.io/crates/direct_ring_buffer)
[![Documentation](https://docs.rs/direct_ring_buffer/badge.svg)](https://docs.rs/direct_ring_buffer)
![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/ain1084/direct_ring_buffer/rust.yml)
![Crates.io License](https://img.shields.io/crates/l/direct_ring_buffer)

This crate provides a high-performance, lock-free ring buffer for single-producer, single-consumer scenarios. The main components of this crate are the `Producer` and `Consumer` structures, which allow for efficient data writing and reading, respectively.

## Overview

A ring buffer is a fixed-size buffer that works as a circular queue. This implementation uses a lock-free approach, making it suitable for real-time applications where minimal latency is crucial. The buffer supports generic types with the `Copy` trait, ensuring that data can be efficiently copied in and out of the buffer.

## Features

- **Single-Producer, Single-Consumer**: Designed for scenarios with a single writer and a single reader.
- **Lock-Free**: Utilizes atomic operations for synchronization, avoiding the need for mutexes or other locking mechanisms.
- **Slice-Based I/O**: Supports reading and writing multiple elements at a time using slices, enhancing performance for bulk operations.
- **Closure-Based Access**: Provides direct access to the buffer through closures, allowing for flexible and efficient data processing.
- **Generic Types with Copy Trait**: The `Copy` trait is not used for copying elements. It is needed to prevent the use of types implementing `Drop`, as the buffer is allocated uninitialized.

## Example

```rust
use direct_ring_buffer::{create_ring_buffer, Producer, Consumer};

let (mut producer, mut consumer) = create_ring_buffer::<u8>(5);

// Writing data to the buffer
producer.write(|data, _| {
    data[..3].copy_from_slice(&[1, 2, 3]);
    3
}, None);

// Reading data from the buffer
consumer.read(|data, _| {
    assert_eq!(data, &[1, 2, 3]);
    data.len()
}, None);
```

In this example, a ring buffer of size 5 is created. The producer writes 3 elements into the buffer, and the consumer reads them back, verifying the data.

## Safety and Performance

This implementation ensures that data is accessed safely through `unsafe` blocks with proper checks. The use of atomic operations ensures minimal overhead for synchronization, making it suitable for high-performance applications.

- **Optimized for Bulk Operations**: Designed to handle multiple elements at once, reducing overhead for batch processing. Single-element operations may incur significant overhead.
