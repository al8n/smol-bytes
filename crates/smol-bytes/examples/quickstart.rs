use smol_bytes::{shared::Bytes, INLINE_CAP};

fn main() {
  let inline: Bytes = Bytes::from_static(b"hello");
  println!("inline = {:?}", inline);

  let heap: Bytes = Bytes::from(vec![42u8; INLINE_CAP + 2]);
  println!("heap len = {}", heap.len());
}
