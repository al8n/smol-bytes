use smol_bytes::SmolBytes;

fn main() {
  let inline = SmolBytes::from_static(b"hello");
  println!("inline = {:?}", inline);

  let heap = SmolBytes::from(vec![42u8; SmolBytes::inline_capacity() + 2]);
  println!("heap len = {}", heap.len());
}
