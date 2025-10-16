#[cfg(test)]
#[cfg(miri)]
const CONCURRENT_TEST_COUNT: usize = 50;

#[cfg(not(miri))]
const CONCURRENT_TEST_COUNT: usize = 50_000;

mod tests {
    use direct_ring_buffer::{create_ring_buffer, Consumer, Producer};
    use rand::Rng;
    use std::{sync::{
        Arc,
        Mutex
     }, thread::{self, JoinHandle}};

    use crate::CONCURRENT_TEST_COUNT;

    #[test]
    fn test_empty() {
        let (p, mut c) = create_ring_buffer::<u8>(10);
        assert_eq!(p.available(), 10);
        assert_eq!(c.available(), 0);
        assert_eq!(c.read_slices(|data, _offset| data.len(), None), 0);
    }

    #[test]
    fn test_empty_write() {
        let (mut p, _c) = create_ring_buffer::<u8>(10);
        assert_eq!(p.write_slices(|_, _| 0, None), 0);
    }

    #[test]
    fn test_empty_read() {
        let (_p, mut c) = create_ring_buffer::<u8>(10);
        assert_eq!(c.read_slices(|_, _| 0, None), 0);
    }

    #[test]
    fn test_full() {
        let (mut p, c) = create_ring_buffer::<u8>(10);
        assert_eq!(
            p.write_slices(
                |data, _offset| {
                    data.iter_mut().enumerate().for_each(|(d, v)| *v = d as u8);
                    10
                },
                None
            ),
            10
        );
        assert_eq!(p.available(), 0);
        assert_eq!(c.available(), 10);
        assert_eq!(p.write_slices(|_data, _offset| 0, None), 0);
    }

    #[test]
    fn test_full_write_and_read() {
        let (mut p, mut c) = create_ring_buffer::<u8>(10);
        assert_eq!(p.write_slices(|data, _offset| data.len(), None), 10);
        assert_eq!(p.available(), 0);
        assert_eq!(c.available(), 10);
        assert_eq!(c.read_slices(|data, _offset| data.len(), None), 10);
        assert_eq!(p.available(), 10);
        assert_eq!(c.available(), 0);
    }

    #[test]
    fn test_element_write() {
        let (mut p, _) = create_ring_buffer::<u8>(5);
        assert!(p.write_element(0));
        assert!(p.write_element(1));
        assert!(p.write_element(2));
        assert!(p.write_element(3));
        assert!(p.write_element(4));
        assert!(!p.write_element(5));
    }

    #[test]
    fn test_element_read() {
        let (mut p, mut c) = create_ring_buffer::<u8>(5);
        assert!(p.write_element(0));
        assert!(p.write_element(1));
        assert!(p.write_element(2));
        assert!(p.write_element(3));
        assert!(p.write_element(4));
        assert!(!p.write_element(5));
        assert_eq!(c.read_element(), Some(0));
        assert_eq!(c.read_element(), Some(1));
        assert_eq!(c.read_element(), Some(2));
        assert_eq!(c.read_element(), Some(3));
        assert_eq!(c.read_element(), Some(4));
        assert_eq!(c.read_element(), None);
    }

    #[test]
    fn test_element_read_write() {
        let (mut p, mut c) = create_ring_buffer::<u8>(5);
        assert!(p.write_element(0));
        assert_eq!(c.read_element(), Some(0));
        assert!(p.write_element(1));
        assert!(p.write_element(2));
        assert_eq!(c.read_element(), Some(1));
        assert!(p.write_element(3));
        assert!(p.write_element(4));
        assert_eq!(c.read_element(), Some(2));
        assert!(p.write_element(5));
        assert_eq!(c.read_element(), Some(3));
        assert!(p.write_element(6));
        assert_eq!(c.read_element(), Some(4));
        assert!(p.write_element(7));
        assert_eq!(c.read_element(), Some(5));
        assert_eq!(c.read_element(), Some(6));
        assert_eq!(c.read_element(), Some(7));
        assert_eq!(c.read_element(), None);
    }

    #[test]
    fn test_write_max_size() {
        let (mut p, mut c) = create_ring_buffer::<u8>(10);
        assert_eq!(
            p.write_slices(
                |data, offset| {
                    assert_eq!(offset, 0);
                    assert_eq!(data.len(), 5);
                    data.copy_from_slice(&[10, 20, 30, 40, 50]);
                    data.len()
                },
                Some(5)
            ),
            5
        );
        assert_eq!(p.available(), 5);
        assert_eq!(c.available(), 5);
        assert_eq!(
            c.read_slices(
                |data, offset| {
                    assert_eq!(offset, 0);
                    assert_eq!(data.len(), 5);
                    assert_eq!(data, &[10, 20, 30, 40, 50]);
                    data.len()
                },
                Some(5)
            ),
            5
        );
    }

    #[test]
    fn test_read_max_size() {
        let (mut p, mut c) = create_ring_buffer::<u8>(10);
        assert_eq!(p.write_slices(|data, _offset| data.len(), None), 10);
        assert_eq!(p.available(), 0);
        assert_eq!(c.available(), 10);
        assert_eq!(c.read_slices(|data, _offset| data.len(), Some(3)), 3);
        assert_eq!(p.available(), 3);
        assert_eq!(c.available(), 7);
    }

    #[test]
    fn test_wraparound() {
        let (mut p, mut c) = create_ring_buffer::<u8>(10);
        assert_eq!(
            p.write_slices(
                |data, offset| {
                    assert_eq!(data.len(), 5);
                    assert_eq!(offset, 0);
                    data.copy_from_slice(&[10, 20, 30, 40, 50]);
                    data.len()
                },
                Some(5),
            ),
            5
        );
        assert_eq!(
            c.read_slices(
                |data, _offset| {
                    assert_eq!(data, &[10, 20, 30]);
                    data.len()
                },
                Some(3)
            ),
            3
        );
        assert_eq!(
            p.write_slices(
                |data, offset| {
                    if offset == 0 {
                        assert_eq!(data.len(), 5);
                        data.copy_from_slice(&[60, 70, 80, 90, 100]);
                    } else {
                        assert_eq!(offset, 5);
                        assert_eq!(data.len(), 3);
                        data.copy_from_slice(&[110, 120, 130]);
                    }
                    data.len()
                },
                None,
            ),
            8,
        );
        assert_eq!(
            c.read_slices(
                |data, offset| {
                    if offset == 0 {
                        assert_eq!(data.len(), 7);
                        assert_eq!(data, &[40, 50, 60, 70, 80, 90, 100]);
                    } else {
                        assert_eq!(offset, 7);
                        assert_eq!(data.len(), 3);
                        assert_eq!(data, &[110, 120, 130]);
                    }
                    data.len()
                },
                None
            ),
            10
        );
    }

    #[test]
    fn test_null_buffer() {
        let (p, c) = create_ring_buffer::<u8>(0);
        assert_eq!(p.available(), 0);
        assert_eq!(c.available(), 0);
    }

    #[test]
    fn test_single_element_buffer() {
        let (mut p, mut c) = create_ring_buffer::<u8>(1);
        assert_eq!(p.available(), 1);
        assert_eq!(c.available(), 0);

        assert_eq!(
            p.write_slices(
                |data, _| {
                    data[0] = 42;
                    1
                },
                None
            ),
            1
        );

        assert_eq!(p.available(), 0);
        assert_eq!(c.available(), 1);

        assert_eq!(
            c.read_slices(
                |data, _| {
                    assert_eq!(data[0], 42);
                    1
                },
                None
            ),
            1
        );

        assert_eq!(p.available(), 1);
        assert_eq!(c.available(), 0);
    }

    #[test]
    fn test_write_break() {
        let (mut p, mut c) = create_ring_buffer::<u8>(10);
        assert_eq!(
            p.write_slices(
                |data, offset| {
                    assert_eq!(data.len(), 10);
                    assert_eq!(offset, 0);
                    data[0..5].copy_from_slice(&[10, 20, 30, 40, 50]);
                    5
                },
                Some(100)
            ),
            5
        );
        assert_eq!(
            p.write_slices(
                |data, offset| {
                    assert_eq!(data.len(), 5);
                    assert_eq!(offset, 0);
                    data.copy_from_slice(&[60, 70, 80, 90, 100]);
                    5
                },
                Some(100)
            ),
            5
        );
        assert_eq!(
            c.read_slices(
                |data, offset| {
                    assert_eq!(offset, 0);
                    assert_eq!(data.len(), 10);
                    assert_eq!(data, &[10, 20, 30, 40, 50, 60, 70, 80, 90, 100]);
                    data.len()
                },
                Some(100)
            ),
            10
        );
    }

    #[test]
    fn test_read_break() {
        let (mut p, mut c) = create_ring_buffer::<u8>(10);
        assert_eq!(
            p.write_slices(
                |data, offset| {
                    assert_eq!(offset, 0);
                    assert_eq!(data.len(), 10);
                    data.copy_from_slice(&[10, 20, 30, 40, 50, 60, 70, 80, 90, 100]);
                    data.len()
                },
                None
            ),
            10
        );
        assert_eq!(
            c.read_slices(
                |data, offset| {
                    assert_eq!(offset, 0);
                    assert_eq!(data.len(), 10);
                    assert_eq!(data, &[10, 20, 30, 40, 50, 60, 70, 80, 90, 100]);
                    5
                },
                None
            ),
            5
        );
        assert_eq!(
            c.read_slices(
                |data, offset| {
                    assert_eq!(offset, 0);
                    assert_eq!(data.len(), 5);
                    assert_eq!(data, &[60, 70, 80, 90, 100]);
                    data.len()
                },
                None
            ),
            5
        );
    }

    #[test]
    fn test_readme_example() {
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
    }

    #[test]
    fn test_slices_read_write() {
        let (mut p, mut c) = create_ring_buffer::<u8>(10);

        fn read_array<T: Copy>(consumer: &mut Consumer<T>, max_len: usize) -> Vec<T> {
            let mut vec = Vec::<T>::new();
            consumer.read_slices(
                |w, _offset| {
                    vec.extend_from_slice(w);
                    w.len()
                },
                Some(max_len),
            );
            vec
        }

        fn write_array<T: Copy>(producer: &mut Producer<T>, buf: &[T]) -> usize {
            producer.write_slices(
                |dest, offset| {
                    dest.copy_from_slice(&buf[offset..offset + dest.len()]);
                    dest.len()
                },
                Some(buf.len()),
            )
        }

        assert_eq!(p.available(), 10);
        assert_eq!(
            write_array(&mut p, &[10, 20, 30, 40, 50, 60, 70, 80, 90, 100]),
            10
        );
        assert_eq!(p.available(), 0);

        assert_eq!(c.available(), 10);
        assert_eq!(read_array(&mut c, 5), &[10, 20, 30, 40, 50]);
        assert_eq!(c.available(), 5);

        assert_eq!(read_array(&mut c, 5), &[60, 70, 80, 90, 100]);
        assert_eq!(c.available(), 0);

        assert_eq!(p.available(), 10);
        assert_eq!(write_array(&mut p, &[10, 20, 30, 40]), 4);
        assert_eq!(p.available(), 6);

        assert_eq!(c.available(), 4);
        assert_eq!(read_array(&mut c, 3), &[10, 20, 30]);
        assert_eq!(c.available(), 1);
        assert_eq!(read_array(&mut c, 2), &[40]);
        assert_eq!(c.available(), 0);
        assert_eq!(read_array(&mut c, 2), &[]);

        assert_eq!(p.available(), 10);
        assert_eq!(write_array(&mut p, &[1, 2, 3, 4, 5]), 5);
        assert_eq!(p.available(), 5);
        assert_eq!(write_array(&mut p, &[6, 7, 8]), 3);
        assert_eq!(p.available(), 2);
        assert_eq!(write_array(&mut p, &[9, 10, 11]), 2);
        assert_eq!(p.available(), 0);

        assert_eq!(c.available(), 10);
        assert_eq!(read_array(&mut c, 8), &[1, 2, 3, 4, 5, 6, 7, 8]);
        assert_eq!(c.available(), 2);

        assert_eq!(p.available(), 8);
        assert_eq!(
            write_array(&mut p, &[21, 22, 23, 24, 25, 26, 27, 28, 29]),
            8
        );
        assert_eq!(p.available(), 0);

        assert_eq!(c.available(), 10);
        assert_eq!(read_array(&mut c, 2), &[9, 10]);
        assert_eq!(c.available(), 8);
        assert_eq!(read_array(&mut c, 3), &[21, 22, 23]);
        assert_eq!(c.available(), 5);
        assert_eq!(read_array(&mut c, 3), &[24, 25, 26]);
        assert_eq!(c.available(), 2);
        assert_eq!(read_array(&mut c, 2), &[27, 28]);
        assert_eq!(c.available(), 0);
    }

    #[test]
    fn test_concurrent_element_read_write() {
        const TEST_COUNT: usize = CONCURRENT_TEST_COUNT;
        let (mut p, mut c) = create_ring_buffer::<usize>(10000);
        let p = thread::spawn(move || {
            for write_value in 0..TEST_COUNT {
                while !p.write_element(write_value) {
                    std::thread::sleep(std::time::Duration::from_millis(1))
                }
            }
        });

        let c = thread::spawn(move || {
            for read_value in 0..TEST_COUNT {
                let rv = loop {
                    if let Some(rv) = c.read_element() {
                        break rv;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1))
                };
                assert_eq!(rv, read_value);
            }
        });

        let _ = p.join();
        let _ = c.join();
    }

    #[test]
    fn test_concurrent_slices_read_write() {
        let (p, c) = create_ring_buffer::<usize>(44100);
        const TEST_COUNT: usize = CONCURRENT_TEST_COUNT;
        const UNIT_MAX: usize = 51;

        fn make_test_thread<F: FnMut(&mut usize, usize) + Send + 'static>(mut f: F) -> JoinHandle<()> {
            let mut count_value = 0usize;
            thread::spawn(move || {
                let mut rng = rand::rng();
                while count_value != TEST_COUNT {
                    let unit = std::cmp::min(rng.random_range(1..UNIT_MAX + 1), TEST_COUNT - count_value);
                    f(&mut count_value, unit);
                }
            })
        }

        let p = Arc::new(Mutex::new(p));
        let p1 = p.clone();
        let writer = make_test_thread(move |write_value, unit| {
            p1.lock().unwrap().write_slices(
                |buf, _offset| {
                    let write_len = buf.len().min(unit);
                    buf[..write_len].iter_mut().for_each(|value| {
                        *value = *write_value;
                        *write_value += 1;
                    });
                    write_len
                },
                Some(unit),
            );
        });

        let c = Arc::new(Mutex::new(c));
        let c1 = c.clone();
        let reader = make_test_thread(move |read_value, unit| {
            c1.lock().unwrap().read_slices(
                |buf, _offset| {
                    let read_len = buf.len().min(unit);
                    buf[..read_len].iter().for_each(|value| {
                        assert_eq!(*value, *read_value);
                        *read_value += 1;
                    });
                    read_len
                },
                Some(unit),
            );
        });
        let _ = writer.join();
        let _ = reader.join();
        assert_eq!(c.lock().unwrap().available(), 0);
        assert_eq!(p.lock().unwrap().available(), 44100);
    }
}
