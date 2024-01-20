use crate::RingBuffer;

#[test]
fn write_buffer_full() {
    let mut buffer = RingBuffer::<3>::new();
    assert_eq!(buffer.write(1), Some(()));
    assert_eq!(buffer.write(2), Some(()));
    assert_eq!(buffer.write(3), None);
    assert_eq!(buffer.read(), Some(1));
    assert_eq!(buffer.write(4), Some(()));
    assert_eq!(buffer.read(), Some(2));
    assert_eq!(buffer.write(5), Some(()));
}

#[test]
fn write_slice_buffer_full() {
    let mut buffer = RingBuffer::<3>::new();
    assert_eq!(buffer.write_slice(&[1, 2, 3]), Err(1));
    assert_eq!(buffer.write_slice(&[1, 2]), Ok(()));
    assert_eq!(buffer.write(4), None);
}

#[test]
fn read_write_wraparound() {
    let mut buffer = RingBuffer::<3>::new();
    for i in 1..10 {
        assert_eq!(buffer.write(i), Some(()));
        assert_eq!(buffer.read(), Some(i));
    }
}
