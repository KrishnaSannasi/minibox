#![no_std]

extern crate alloc;

use core::sync::atomic::{AtomicBool, Ordering::SeqCst};
use minibox::MiniBox;
use static_alloc::Bump;

pub struct PanicOnAlloc(Bump<[u8; 1 << 16]>);

static FLAG: AtomicBool = AtomicBool::new(false);

use alloc::alloc::{GlobalAlloc, Layout};
unsafe impl GlobalAlloc for PanicOnAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        if FLAG.load(SeqCst) {
            panic!("tried to allocate in a noalloc test")
        }

        GlobalAlloc::alloc(&self.0, layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        if FLAG.load(SeqCst) {
            panic!("tried to allocate in a noalloc test")
        }

        GlobalAlloc::dealloc(&self.0, ptr, layout)
    }
}

#[global_allocator]
static A: PanicOnAlloc = PanicOnAlloc(Bump::uninit());

fn with<F: FnOnce()>(f: F) {
    struct OnDrop;

    impl Drop for OnDrop {
        fn drop(&mut self) {
            FLAG.store(false, SeqCst);
        }
    }

    assert!(!FLAG.swap(true, SeqCst));
    let _on_drop = OnDrop;

    f()
}

#[test]
fn noalloc() {
    with(|| {
        #[repr(align(64))]
        struct OverAlignedZeroSized;

        MiniBox::new([10_u8; 4]);
        MiniBox::new(());
        MiniBox::new(OverAlignedZeroSized);
    })
}
