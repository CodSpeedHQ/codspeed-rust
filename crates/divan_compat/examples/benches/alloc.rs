use std::{
    alloc::Layout,
    collections::{HashMap, HashSet},
};

#[divan::bench]
fn allocate() {
    println!("Hello, world!");

    let vec = vec![1, 2, 3];
    println!("{vec:?}");

    let mut map = HashMap::new();
    map.insert("key", "value");
    println!("{map:?}");

    let mut set = HashSet::new();
    set.insert("apple");
    set.insert("banana");
    println!("{set:?}");

    std::thread::sleep(std::time::Duration::from_secs(1));

    let mut bytes_vec = vec![0u8; 0x100];
    println!("{:?}", bytes_vec.len());

    bytes_vec.extend(&vec![0u8; 0x1000]);

    // Alloc 42 bytes of memory per iteration (4200 bytes total)
    for _ in 0..100 {
        let layout = Layout::new::<[u8; 42]>();
        let memory = unsafe { std::alloc::alloc(layout) };
        core::hint::black_box(memory);
        unsafe { std::alloc::dealloc(memory, layout) };
    }
}

fn main() {
    divan::main();
}
