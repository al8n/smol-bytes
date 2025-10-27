use bytes::{BufMut, Bytes, BytesMut};

#[test]
fn t() {
  println!("{}", size_of::<BytesMut>());
  println!("{}", size_of::<Bytes>());
}
