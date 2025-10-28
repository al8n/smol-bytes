use smol_bytes::{strategy::shared::SmolBytes, INLINE_CAP};

fn main() {
  let inline: SmolBytes = SmolBytes::from_static(b"hello");
  println!("inline = {:?}", inline);

  let heap: SmolBytes = SmolBytes::from(vec![42u8; INLINE_CAP + 2]);
  println!("heap len = {}", heap.len());
}
