# TASK

You are a Principal Rust soft engineer and good at Rust code design, now, please help me finish the remaining tasks.

You can create folders and files under the smol-bytes project.

Wrappers around `Buffer`, `Bytes` and `BytesMut` respectively,  providing a String-like interface. They allow splitting into owned sub-slices, similar to how the `Buffer`, `Bytes` and `BytesMut` work.

1. Implement a wrapper over Buffer called Utf8Buffer in utf8_buffer.rs
   1. push, push a char to the buffer
   2. push_str, push a str to the buffer
   3. split_to, similar to Buffer::split_to, but panic on char boundary
   4. try_split_to, similar to Buffer::try_split_to, but check the char boundary and return an error report it is a char boundary
   5. split_off, similar to Buffer::split_off, but panic on char boundary
   6. try_split_off, similar to Buffer::try_split_off, but check the char boundary and return an error report it is a char boundary
   7. slice, similar to Buffer::slice, but panic the char boundary.
   8. try_slice, similar to Buffer::try_slice, but check the char boundary and return an error report it is a char boundary
   9. comparison operations, reference the buffer/cmp.rs
   10. from operations, refernece the buffer/from.rs
   11. ops, reference the buffer/ops.rs
   12. arbitray, quickcheck, borsh, serde, python impl, reference the buffer/*
2. Implement a wrapper over RawBytes called Utf8RawBytes in utf8_bytes.rs
   1. similar to Utf8Buffer, reference the bytes/*
3. Implement a wrapper over BytesMut called Utf8BytesMut in utf8_bytes_mut.rs
   1. similar to Utf8Buffer, reference the bytes_mut/*

And everything you think are missing for a professional project.
