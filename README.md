# Direct Ring Buffer

[![Crates.io](https://img.shields.io/crates/v/direct_ring_buffer.svg)](https://crates.io/crates/direct_ring_buffer)
[![Documentation](https://docs.rs/direct_ring_buffer/badge.svg)](https://docs.rs/direct_ring_buffer)
[![Build Status](https://github.com/ain1084/direct_ring_buffer/workflows/Rust/badge.svg)](https://github.com/ain1084/direct_ring_buffer/actions?query=workflow%3ARust)
![Crates.io License](https://img.shields.io/crates/l/direct_ring_buffer)

A high-performance, lock-free ring buffer for Rust.  
Designed for safe, efficient bulk data transfer — ideal for real-time systems such as **audio streaming** or **signal processing**.

---

## Features

- **Lock-free** design using atomic operations (single-producer / single-consumer)  
- **Full-capacity utilization** — all allocated elements are usable  
- **Stable wrap-around alignment** for predictable block-based I/O  
- **Slice-based read/write API** for high-throughput contiguous access  
- **Thread-safe** operation without explicit locking  
- **No heap allocation during runtime** — only during buffer creation  
- **Supports `Drop` types** safely (`Vec`, `String`, etc.)  
- **Miri-verified** — passes [Miri](https://github.com/rust-lang/miri) with no undefined behavior  

---

## Overview

A **ring buffer** (also known as a circular buffer) provides a fixed-size queue that efficiently reuses memory for continuous read/write operations.

This crate differs from many common ring buffer implementations in several key ways:

- It allows the **entire buffer capacity** to be used — no slot is reserved to distinguish full/empty states.  
  For example, a buffer created with a capacity of 2048 elements can actually hold all 2048.
- When reading and writing in consistent block sizes,  
  the wrap-around point remains **stable and predictable**, which is especially valuable for **audio and streaming systems**.
- It focuses on **slice-based operations** rather than element-by-element methods like `push` / `pop`,  
  enabling bulk data movement and better cache locality.

All internal elements are safely initialized using `T::default()`,  
eliminating undefined behavior (UB) while maintaining performance comparable to traditional uninitialized approaches.

---

## Type Constraints

The type parameter `T` no longer needs to be `Copy` for the buffer itself.  
All internal elements are fully initialized at creation time using `T::default()`.

Operation-level requirements:

- **`read_element` / `write_element`** — require `T: Copy`, since they move values by copy.  
- **`read_slices` / `write_slices`** — do *not* require `Copy`; they operate directly on initialized memory.

This enables both high-performance numeric buffers and safe use of complex types.

---

## Examples

### Minimal (no wrap-around; ignore offset)

```rust
use direct_ring_buffer::create_ring_buffer;

let (mut producer, mut consumer) = create_ring_buffer::<u8>(5);

// Write 4 elements in one contiguous region.
// We know wrap-around won't happen, so we ignore `offset`.
producer.write_slices(|buf, offset| {
    assert_eq!(offset, 0);
    buf[..4].copy_from_slice(&[1, 2, 3, 4]);
    4 // number of elements written
}, None);

// Read back the same 4 elements in one contiguous region.
// Again, no wrap-around here; `offset` is irrelevant.
consumer.read_slices(|buf, offset| {
    assert_eq!(offset, 0);
    assert_eq!(&buf[..4], &[1, 2, 3, 4]);
    4 // number of elements read
}, None);
```

### Wrap-Around

```rust
use direct_ring_buffer::create_ring_buffer;

let (mut producer, mut consumer) = create_ring_buffer::<u8>(5);

// Prep: fill 4 slots so the head/tail positions make the next write split.
// Then consume only 3, leaving 1 element inside to force a wrap on the next write.
producer.write_slices(|buf, _| {
    buf[..4].copy_from_slice(&[1, 2, 3, 4]);
    4
}, None);
consumer.read_slices(|buf, _| {
    assert_eq!(&buf[..3], &[1, 2, 3]);
    3
}, None);
// Buffer now holds a single element `[4]` near the end; the next write will wrap.

// Write 3 elements in **one** call. The closure may be invoked twice:
// - first for the tail end (offset = 0, write 1 element),
// - second for the wrapped beginning (offset = 1, write 2 elements).
let write_seq = [6, 7, 8];
producer.write_slices(|buf, offset| {
    // `offset` is the linear position (in elements) already written in THIS call.
    let n = buf.len().min(write_seq.len() - offset);
    buf[..n].copy_from_slice(&write_seq[offset..offset + n]);
    n
}, None);

// Read back all 4 elements now in the buffer: `[4, 6, 7, 8]`.
// This read may also split into two contiguous chunks (end then start).
let mut out = [0u8; 4];
consumer.read_slices(|buf, offset| {
    // `offset` is the linear position already read in THIS call.
    let n = buf.len().min(out.len() - offset);
    out[offset..offset + n].copy_from_slice(&buf[..n]);
    n
}, None);

assert_eq!(out, [4, 6, 7, 8]);
```

💬 **Notes**

- The closure can be called up to twice per operation (once for the tail, once for the wrapped head).
- On the first invocation offset == 0; if a second invocation occurs, offset equals the number of elements processed in the first invocation.
- Always return the actual number of elements written/read (≤ buf.len()); returning less stops the operation early.

### Single-element

```rust
use direct_ring_buffer::create_ring_buffer;

let (mut producer, mut consumer) = create_ring_buffer::<u8>(4);

// Step 1: Write 4 elements (fill the buffer)
for i in 1..=4 {
    assert!(producer.write_element(i));
}

// Step 2: Read 2 elements
for expected in [1, 2] {
    let value = consumer.read_element().unwrap();
    assert_eq!(value, expected);
}

// Step 3: Write 2 more elements (these will wrap around internally)
for i in [5, 6] {
    assert!(producer.write_element(i));
}

// Step 4: Read all 4 elements now in the buffer
let mut result = Vec::new();
while let Some(v) = consumer.read_element() {
    result.push(v);
}

assert_eq!(result, vec![3, 4, 5, 6]);
```

💡 **Explanation**

- write_element() writes a single element if space is available and returns true on success.
- read_element() reads one element if available and returns Some(T).
- The example shows:
  1. Fill the buffer ([1, 2, 3, 4])
  2. Consume 2 elements ([1, 2])
  3. Add 2 new elements ([5, 6]) — this triggers wrap-around internally
- Read the remaining 4 elements in order ([3, 4, 5, 6])

---

## Safety and Performance

All internal memory is safely initialized — no uninitialized or dangling references are used anywhere.  
Atomic operations ensure consistent synchronization between producer and consumer threads without locks.

The code internally uses a small number of `unsafe` blocks,  
but every instance has been validated to conform to Rust’s aliasing and ownership rules.

This implementation has been verified to pass **Miri**,  
confirming that it exhibits **no undefined behavior under Rust's memory model**.

Performance remains comparable or better than the prior uninitialized version,  
since initialization with `Default` occurs only once during construction.

---

## Notes

This crate was refactored to remove all unsafe uninitialized memory handling  
and is now fully compliant with Miri and Rust’s strict aliasing model.

While previous versions worked in practice,  
this implementation guarantees correctness by design — both in safe and `unsafe` contexts.

Its slice-oriented API and full-capacity utilization make it ideal for real-time streaming workloads,  
where stable boundaries and predictable throughput are essential.

---

## License

Licensed under either of

- Apache License, Version 2.0
  ([LICENSE-APACHE](LICENSE-APACHE) or <http://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
  at your option.

---

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
