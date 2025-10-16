#![doc = include_str!("../README.md")]

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

    /// Writes elements to the ring buffer.
    ///
    /// This method writes elements to the ring buffer using the provided closure.
    /// The closure `f` receives a mutable slice of writable elements and the
    /// current offset within the write operation, and it should return the number
    /// of elements written. The `max_size` parameter specifies the maximum number of
    /// elements to write. If `None`, the method attempts to write as many elements
    /// as available.
    ///
    /// If there is no space available for writing, the function returns immediately
    /// without blocking, and the closure is not called.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure for writing elements. It takes a mutable slice of writable
    ///   elements and an offset, and returns the number of elements written. The
    ///   closure will not be called if there are no writable elements. If the
    ///   buffer wraps around, the closure may be called twice. The slice passed
    ///   to the closure contains the currently writable elements. The offset is
    ///   `0` for the first call and increases by the number of elements written
    ///   in subsequent calls. If the closure returns a value less than the
    ///   length of the slice passed to it, it is considered as an interruption
    ///   of the write operation by that number of elements.
    /// * `max_size` - An optional parameter specifying the maximum number of elements
    ///   to write. If `None`, the method will write up to the number of
    ///   available elements.
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
    /// producer.write_slices(|data, _| {
    ///     data[..3].copy_from_slice(&[1, 2, 3]);
    ///     3
    /// }, None);
    ///
    /// producer.write_slices(|data, _| {
    ///     data[..2].copy_from_slice(&[4, 5]);
    ///     2
    /// }, None);
    /// assert_eq!(producer.available(), 0);
    /// ```
    pub fn write_slices(
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

    /// Writes elements to the ring buffer. (Deprecated)
    ///
    /// This method writes elements to the ring buffer using the provided closure.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure for writing elements.
    /// * `max_size` - An optional parameter specifying the maximum number of elements to write.
    ///
    /// # Returns
    ///
    /// The number of elements written.
    #[deprecated(note = "Please use `write_slices` instead")]
    pub fn write(
        &mut self,
        f: impl FnMut(&mut [T], usize) -> usize,
        max_size: Option<usize>,
    ) -> usize {
        self.write_slices(f, max_size)
    }

    /// Writes a single element to the ring buffer.
    ///
    /// This method writes a single element to the ring buffer. If the buffer is full,
    /// it returns `false`.
    ///
    /// # Arguments
    ///
    /// * `value` - The element to write to the buffer.
    ///
    /// # Returns
    ///
    /// `true` if the element was successfully written, `false` if the buffer is full.
    ///
    /// # Example
    ///
    /// ```
    /// use direct_ring_buffer::{create_ring_buffer};
    ///
    /// let (mut producer, mut consumer) = create_ring_buffer::<u8>(5);
    /// assert!(producer.write_element(1));
    /// assert!(producer.write_element(2));
    /// assert!(producer.write_element(3));
    /// assert!(producer.write_element(4));
    /// assert!(producer.write_element(5));
    /// assert!(!producer.write_element(6)); // Buffer is full
    /// assert_eq!(producer.available(), 0);
    /// assert_eq!(consumer.available(), 5);
    /// consumer.read_slices(|data, offset| {
    ///     assert_eq!(data, &([1, 2, 3, 4, 5][offset..offset + data.len()]));
    ///     data.len()
    /// }, None);
    /// assert_eq!(consumer.available(), 0);
    /// assert_eq!(producer.available(), 5);
    /// ```
    pub fn write_element(&mut self, value: T) -> bool {
        self.buffer.write_element(&mut self.index, value)
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

    /// Reads elements from the ring buffer.
    ///
    /// This method reads elements from the ring buffer using the provided closure.
    /// The closure `f` receives a slice of readable elements and the current
    /// offset within the read operation, and it should return the number of elements
    /// read. The `max_size` parameter specifies the maximum number of elements to
    /// read. If `None`, the method attempts to read as many elements as available.
    ///
    /// If there are no elements available for reading, the function returns
    /// immediately without blocking, and the closure is not called.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that processes the readable elements. It takes a reference
    ///   to a slice of readable elements and an offset as arguments, and
    ///   returns the number of elements read. The closure will not be called if
    ///   there are no readable elements. If the buffer wraps around, the closure
    ///   may be called twice. The slice passed to the closure contains the
    ///   currently accessible elements. The offset is `0` for the first call
    ///   and increases by the number of elements read in subsequent calls. If
    ///   the closure returns a value less than the length of the slice passed to
    ///   it, it is considered as an interruption of the read operation by that
    ///   number of elements.
    /// * `max_size` - An optional parameter specifying the maximum number of elements
    ///   to read. If `None`, the method will read up to the number of
    ///   available elements.
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
    /// producer.write_slices(|data, offset| {
    ///     assert_eq!(data.len(), 5);
    ///     data[..2].copy_from_slice(&[1, 2]);
    ///     2
    /// }, None);
    /// consumer.read_slices(|data, offset| {
    ///     assert_eq!(data.len(), 2);
    ///     assert_eq!(offset, 0);
    ///     2
    /// }, None);
    /// producer.write_slices(|data, offset| {
    ///     data.copy_from_slice(&([3, 4, 5, 6, 7][offset..offset + data.len()]));
    ///     data.len()
    /// }, None);
    /// consumer.read_slices(|data, offset| {
    ///     assert_eq!(data, &([3, 4, 5, 6, 7][offset..offset + data.len()]));
    ///     data.len()
    /// }, None);
    ///
    /// ```
    pub fn read_slices(
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

    /// Reads elements from the ring buffer. (Deprecated)
    ///
    /// This method reads elements from the ring buffer using the provided closure.
    ///
    /// # Arguments
    ///
    /// * `f` - A closure that processes the readable elements.
    /// * `max_size` - An optional parameter specifying the maximum number of elements to read.
    ///
    /// # Returns
    ///
    /// The number of elements read.
    #[deprecated(note = "Please use `read_slices` instead")]
    pub fn read(
        &mut self,
        f: impl FnMut(&[T], usize) -> usize,
        max_size: Option<usize>,
    ) -> usize {
        self.read_slices(f, max_size)
    } 
   
    /// Reads a single element from the ring buffer.
    ///
    /// This method reads a single element from the ring buffer and returns it. If the
    /// buffer is empty, it returns `None`.
    ///
    /// # Returns
    ///
    /// An `Option` containing the element if available, or `None` if the buffer is
    /// empty.
    ///
    /// # Example
    ///
    /// ```
    /// use direct_ring_buffer::{create_ring_buffer};
    ///
    /// let (mut producer, mut consumer) = create_ring_buffer::<u8>(5);
    /// producer.write_slices(|data, offset| {
    ///     data.copy_from_slice(&([3, 4, 5, 6, 7][offset..offset + data.len()]));
    ///     data.len()
    /// }, None);
    /// assert_eq!(consumer.read_element(), Some(3));
    /// assert_eq!(consumer.read_element(), Some(4));
    /// assert_eq!(consumer.read_element(), Some(5));
    /// assert_eq!(consumer.read_element(), Some(6));
    /// assert_eq!(consumer.read_element(), Some(7));
    /// assert_eq!(consumer.read_element(), None);
    /// ```
    pub fn read_element(&mut self) -> Option<T> where T: Copy {
        self.buffer.read_element(&mut self.index)
    }
}

unsafe impl<T> Send for Consumer<T> {}

struct DirectRingBuffer<T> {
    elements: UnsafeCell<Box<[T]>>,
    used: AtomicUsize,
}

impl<T> DirectRingBuffer<T> {
    /// Returns the number of elements available for reading.
    #[inline]
    fn available_read(&self) -> usize {
        self.used.load(Ordering::Acquire)
    }

    /// Returns the number of elements available for writing.
    #[inline]
    fn available_write(&self) -> usize {
        self.elements().len() - self.used.load(Ordering::Acquire)
    }

    /// Returns a mutable reference to the elements the buffer.
    #[inline]
    #[allow(clippy::mut_from_ref)]
    fn elements(&self) -> &mut Box<[T]> {
        unsafe { &mut *self.elements.get() }
    }

    /// Updates the index to wrap around the buffer.
    #[inline]
    fn wraparound_index(&self, index: &mut usize, advance: usize) {
        *index = if *index + advance >= self.elements().len() {
            0
        } else {
            *index + advance
        }
    }

    /// Reads a single element from the buffer.
    fn read_element(&self, index: &mut usize) -> Option<T> where T: Copy {
        if self.available_read() == 0 {
            None
        } else {
            let ret = Some(self.elements()[*index]);
            self.wraparound_index(index, 1);
            self.used.fetch_sub(1, Ordering::Release);
            ret
        }
    }

    /// Writes a single element to the buffer.
    fn write_element(&self, index: &mut usize, value: T) -> bool {
        if self.available_write() == 0 {
            false
        } else {
            self.elements()[*index] = value;
            self.wraparound_index(index, 1);
            self.used.fetch_add(1, Ordering::Release);
            true
        }
    }

    /// Read/Write common process.
    fn process_slices(
        &self,
        index: &mut usize,
        available: usize,
        mut f: impl FnMut(*mut T, usize, usize) -> usize,
        max_size: Option<usize>,
        update_used: impl FnOnce(&AtomicUsize, usize),
    ) -> usize {
        let elements = self.elements();
        let elements_len = elements.len();
        let mut total_processed = 0;
        let max_size = max_size.unwrap_or(available).min(available);

        while total_processed < max_size {
            let part_start = *index;
            let part_len = (elements_len - part_start).min(max_size - total_processed);
            let processed = f(
                unsafe { elements.get_unchecked_mut(part_start) },
                part_len,
                total_processed,
            );
            total_processed += processed;
            self.wraparound_index(index, processed);
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
/// producer.write_slices(|data, _| {
///     data.copy_from_slice(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
///     10
/// }, None);
///
/// let mut read_data = vec![0; 10];
/// consumer.read_slices(|data, _| {
///     read_data[..data.len()].copy_from_slice(data);
///     data.len()
/// }, None);
/// assert_eq!(read_data, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10]);
/// ```
#[allow(clippy::uninit_vec)]
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
