#![no_std]

use minibox::MiniBox;
use static_alloc::Bump;

#[global_allocator]
static A: Bump<[u8; 1 << 16]> = Bump::uninit();

#[test]
fn smoke() {
    let _bx = MiniBox::new([10_u32; 16]);
}
