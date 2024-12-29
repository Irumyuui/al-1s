#![feature(allocator_api)]
#![feature(portable_simd)]

pub mod utils;

use mimalloc_rust::GlobalMiMalloc;

#[global_allocator]
static GLOBAL_ALLOCATOR: GlobalMiMalloc = GlobalMiMalloc;

extern crate static_assertions as sa;

#[ctor::ctor]
fn _init() {
    color_backtrace::install();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_file(true)
        .with_line_number(true)
        .init();
}
