/*!
# Direct Ring Buffer

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

## Safety and Performance

This implementation ensures that data is accessed safely through `unsafe` blocks with
proper checks. The use of atomic operations ensures minimal overhead for
synchronization, making it suitable for high-performance applications.

## Optimized for Bulk Operations

Designed to handle multiple elements at once, reducing overhead for batch processing.
Single-element operations may incur significant overhead.
*/

use std::{
    cell::UnsafeCell,
    slice::{from_raw_parts, from_raw_parts_mut},
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
};

/// Producer part of the ring buffer.
pub struct Producer<T> {
    buffer: Arc<DirectRingBuffer<T>>,
    index: usize,
}

impl<T> Producer<T> {
    /// Returns the number of elements available for writing.
    ///
    /// This method returns the number of elements available for writing.
    ///
    /// # Returns
    ///
    /// Number of elements available for writing.
    ///
    /// # Example
    ///
    /// ```
    /// use direct_ring_buffer::{create_ring_buffer};
    ///
    /// let (producer, _) = create_ring_buffer::<u8>(5);
    /// assert_eq!(producer.available(), 5);
    /// ```
    pub fn available(&self) -> usize {
        self.buffer.available_write()
    }

    /// Writes data to the ring buffer.
    ///
    /// This method writes data to the ring buffer using the provided closure.
    /// The closure `f` receives a mutable slice of writable elements and the
    /// current offset within the write operation, and it should return the number
    /// of elements written. The `max_size` parameter specifies the maximum number of
    /// elements to write. If `None`, the method attempts to write as many elements as
    /// available.
    ///
    /// If there is no space available for writing, the function returns immediately
    /// without blocking, and the closure is not called.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure for writing elements. It takes a mutable slice of
    ///         writable elements and an offset, and returns the number of
    ///         elements written. The closure will not be called if there are no
    ///         writable elements. If the buffer wraps around, the closure may be
    ///         called twice. The slice passed to the closure contains the
    ///         currently writable elements. The offset is `0` for the first call
    ///         and increases by the number of elements written in subsequent calls.
    ///         If the closure returns a value less than the length of the slice passed
    ///         to it, it is considered as an interruption of the write operation by
    ///         that number of elements.
    /// * `max_size` - An optional parameter specifying the maximum number of
    ///                elements to write. If `None`, the method will write up to
    ///                the number of available elements.
    ///
    /// # Returns
    ///
    /// The number of elements written.
    ///
    /// # Example
    ///
    /// ```
    /// use direct_ring_buffer::{create_ring_buffer, Producer};
    ///
    /// let (mut producer, _) = create_ring_buffer::<u8>(5);
    /// producer.write(|data, _| {
    ///     data[..3].copy_from_slice(&[1, 2, 3]);
    ///     3
    /// }, None);
    ///
    /// producer.write(|data, _| {
    ///     data[..2].copy_from_slice(&[4, 5]);
    ///     2
    /// }, None);
    /// assert_eq!(producer.available(), 0);
    /// ```
    pub fn write(
        &mut self,
        mut f: impl FnMut(&mut [T], usize) -> usize,
        max_size: Option<usize>,
    ) -> usize {
        let available = self.available();
        self.buffer.process_slices(
            &mut self.index,
            available,
            |buf, len, process_offset| {
                f(
                    // No boundaries are crossed.
                    unsafe { from_raw_parts_mut(buf, len) },
                    process_offset,
                )
            },
            max_size,
            |atomic, processed| {
                atomic.fetch_add(processed, Ordering::Release);
            },
        )
    }
}

unsafe impl<T> Send for Producer<T> {}

/// Consumer part of the ring buffer.
pub struct Consumer<T> {
    buffer: Arc<DirectRingBuffer<T>>,
    index: usize,
}

impl<T> Consumer<T> {
    /// Returns the number of elements available for reading.
    ///
    /// This method returns the number of elements available for reading.
    ///
    /// # Returns
    ///
    /// Number of elements available for reading.
    ///
    /// # Example
    /// ```
    /// use direct_ring_buffer::{create_ring_buffer};
    ///
    /// let (_, consumer) = create_ring_buffer::<u8>(5);
    /// assert_eq!(consumer.available(), 0);
    /// ```
    pub fn available(&self) -> usize {
        self.buffer.available_read()
    }

    /// Reads data from the ring buffer.
    ///
    /// This method reads data from the ring buffer using the provided closure.
    /// The closure `f` receives a slice of readable elements and the current
    /// offset within the read operation, and it should return the number of elements
    /// read. The `max_size` parameter specifies the maximum number of elements to
    /// read. If `None`, the method attempts to read as many elements as available.
    ///
    /// If there is no data available for reading, the function returns immediately
    /// without blocking, and the closure is not called.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that processes the readable elements. It takes a
    ///         reference to a slice of readable elements and an offset as
    ///         arguments, and returns the number of elements read. The closure
    ///         will not be called if there are no readable elements. If the
    ///         buffer wraps around, the closure may be called twice. The slice
    ///         passed to the closure contains the currently accessible elements.
    ///         The offset is `0` for the first call and increases by the number
    ///         of elements read in subsequent calls.
    ///         If the closure returns a value less than the length of the slice
    ///         passed to it,
    ///         it is considered as an interruption of the read operation by that
    ///         number of elements.
    /// * `max_size` - An optional parameter specifying the maximum number of
    ///                elements to read. If `None`, the method will read up to
    ///                the number of available elements.
    ///
    /// # Returns
    ///
    /// The number of elements read.
    ///
    /// # Example
    ///
    /// ```
    /// use direct_ring_buffer::{create_ring_buffer};
    ///
    /// let (mut producer, mut consumer) = create_ring_buffer::<u8>(5);
    /// producer.write(|data, offset| {
    ///     assert_eq!(data.len(), 5);
    ///     data[..2].copy_from_slice(&[1, 2]);
    ///     2
    /// }, None);
    /// consumer.read(|data, offset| {
    ///     assert_eq!(data.len(), 2);
    ///     assert_eq!(offset, 0);
    ///     2
    /// }, None);
    /// producer.write(|data, offset| {
    ///     data.copy_from_slice(&([3, 4, 5, 6, 7][offset..offset + data.len()]));
    ///     data.len()
    /// }, None);
    /// consumer.read(|data, offset| {
    ///     assert_eq!(data, &([3, 4, 5, 6, 7][offset..offset + data.len()]));
    ///     data.len()
    /// }, None);
    ///
    /// ```
    pub fn read(
        &mut self,
        mut f: impl FnMut(&[T], usize) -> usize,
        max_size: Option<usize>,
    ) -> usize {
        let available = self.available();
        self.buffer.process_slices(
            &mut self.index,
            available,
            |buf, len, process_offset| {
                f(
                    // No boundaries are crossed.
                    unsafe { from_raw_parts(buf, len) },
                    process_offset,
                )
            },
            max_size,
            |atomic, processed| {
                atomic.fetch_sub(processed, Ordering::Release);
            },
        )
    }
}

unsafe impl<T> Send for Consumer<T> {}

struct DirectRingBuffer<T> {
    elements: UnsafeCell<Box<[T]>>,
    used: AtomicUsize,
}

impl<T> DirectRingBuffer<T> {
    /// Returns the number of elements available for reading.
    fn available_read(&self) -> usize {
        self.used.load(Ordering::Acquire)
    }

    /// Returns the number of elements available for writing.
    fn available_write(&self) -> usize {
        unsafe { &*self.elements.get() }.len() - self.used.load(Ordering::Acquire)
    }

    /// Read/Write common process (internal).
    fn process_slices(
        &self,
        index: &mut usize,
        available: usize,
        mut f: impl FnMut(*mut T, usize, usize) -> usize,
        max_size: Option<usize>,
        update_used: impl FnOnce(&AtomicUsize, usize),
    ) -> usize {
        let buffer = unsafe { &mut *self.elements.get() };
        let buffer_len = buffer.len();
        let mut total_processed = 0;
        let max_size = max_size.unwrap_or(available).min(available);

        while total_processed < max_size {
            let part_start = *index;
            let part_len = (buffer_len - part_start).min(max_size - total_processed);
            let processed = f(
                unsafe { buffer.get_unchecked_mut(part_start) },
                part_len,
                total_processed,
            );
            total_processed += processed;
            *index += processed;
            if *index >= buffer_len {
                *index = 0
            }
            if processed < part_len {
                // Aborting the operation because the return value
                // from the closure is smaller then expected.
                break;
            }
        }
        update_used(&self.used, total_processed);
        total_processed
    }
}

/// Creates a ring buffer with the specified size.
///
/// # Arguments
///
/// * `size` - The size of the ring buffer.
///
/// # Returns
///
/// A tuple containing a `Producer<T>` and a `Consumer<T>`.
///
/// # Example
///
/// ```
/// use direct_ring_buffer::create_ring_buffer;
/// let (mut producer, mut consumer) = create_ring_buffer::<u8>(10);
/// producer.write(|data, _| {
///     data.copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
///     10
/// }, None);
///
/// let mut read_data = vec![0; 10];
/// consumer.read(|data, _| {
///     read_data[..data.len()].copy_from_slice(data);
///     data.len()
/// }, None);
/// assert_eq!(read_data, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
/// ```
pub fn create_ring_buffer<T: Copy>(size: usize) -> (Producer<T>, Consumer<T>) {
    let buffer = Arc::new(DirectRingBuffer {
        elements: UnsafeCell::new({
            let mut vec = Vec::<T>::with_capacity(size);
            unsafe { vec.set_len(size) };
            vec.into_boxed_slice()
        }),
        used: AtomicUsize::new(0),
    });
    (
        Producer {
            buffer: Arc::clone(&buffer),
            index: 0,
        },
        Consumer { buffer, index: 0 },
    )
}
