#[cfg(test)]
mod tests {
    use direct_ring_buffer::{create_ring_buffer, Consumer, Producer};
    use rand::Rng;
    use std::thread;

    #[test]
    fn test_empty() {
        let (p, mut c) = create_ring_buffer::<u8>(10);
        assert_eq!(p.available(), 10);
        assert_eq!(c.available(), 0);
        assert_eq!(c.read(|data, _offset| data.len(), None), 0);
    }

    #[test]
    fn test_empty_read() {
        let (_p, mut c) = create_ring_buffer::<u8>(10);
        assert_eq!(c.read(|_, _| 0, None), 0);
    }

    #[test]
    fn test_full() {
        let (mut p, c) = create_ring_buffer::<u8>(10);
        assert_eq!(
            p.write(
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
        assert_eq!(p.write(|_data, _offset| 0, None), 0);
    }

    #[test]
    fn test_full_write_and_read() {
        let (mut p, mut c) = create_ring_buffer::<u8>(10);
        assert_eq!(p.write(|data, _offset| data.len(), None), 10);
        assert_eq!(p.available(), 0);
        assert_eq!(c.available(), 10);
        assert_eq!(c.read(|data, _offset| data.len(), None), 10);
        assert_eq!(p.available(), 10);
        assert_eq!(c.available(), 0);
    }

    #[test]
    fn test_write_max_size() {
        let (mut p, mut c) = create_ring_buffer::<u8>(10);
        assert_eq!(
            p.write(
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
            c.read(
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
        assert_eq!(p.write(|data, _offset| data.len(), None), 10);
        assert_eq!(p.available(), 0);
        assert_eq!(c.available(), 10);
        assert_eq!(c.read(|data, _offset| data.len(), Some(3)), 3);
        assert_eq!(p.available(), 3);
        assert_eq!(c.available(), 7);
    }

    #[test]
    fn test_wraparound() {
        let (mut p, mut c) = create_ring_buffer::<u8>(10);
        assert_eq!(
            p.write(
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
            c.read(
                |data, _offset| {
                    assert_eq!(data, &[10, 20, 30]);
                    data.len()
                },
                Some(3)
            ),
            3
        );
        assert_eq!(
            p.write(
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
            c.read(
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
            p.write(
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
            c.read(
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
            p.write(
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
            p.write(
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
            c.read(
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
            p.write(
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
            c.read(
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
            c.read(
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
    fn test_read_write() {
        let (mut p, mut c) = create_ring_buffer::<u8>(10);

        fn read_array<T: Copy>(consumer: &mut Consumer<T>, max_len: usize) -> Vec<T> {
            let mut vec = Vec::<T>::new();
            consumer.read(
                |w, _offset| {
                    vec.extend_from_slice(w);
                    w.len()
                },
                Some(max_len),
            );
            vec
        }

        fn write_array<T: Copy>(producer: &mut Producer<T>, buf: &[T]) -> usize {
            producer.write(
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
    fn test_concurrent_read_write() {
        let (mut p, mut c) = create_ring_buffer::<usize>(44100);
        const TEST_LIMIT: usize = 50_000_000;
        const UNIT_MAX: usize = 51;

        let mut write_value = 0usize;
        let p = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            while write_value != TEST_LIMIT {
                let unit = std::cmp::min(rng.gen_range(1..UNIT_MAX + 1), TEST_LIMIT - write_value);
                let _ = p.write(
                    |buf: &mut [usize], _offset| {
                        buf.iter_mut().for_each(|value| {
                            *value = write_value;
                            write_value += 1;
                        });
                        buf.len()
                    },
                    Some(unit),
                );
            }
        });

        let mut read_value = 0usize;
        let c = thread::spawn(move || {
            let mut rng = rand::thread_rng();
            while read_value != TEST_LIMIT {
                let unit = std::cmp::min(rng.gen_range(1..UNIT_MAX + 1), TEST_LIMIT - read_value);
                let _ = c.read(
                    |buf, _offset| {
                        buf.iter().for_each(|value| {
                            assert_eq!(*value, read_value);
                            read_value += 1;
                        });
                        buf.len()
                    },
                    Some(unit),
                );
            }
            assert_eq!(c.available(), 0);
        });
        let _ = p.join();
        let _ = c.join();
    }
}
