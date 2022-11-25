use std::arch::asm;

pub type Value = u64;

#[inline(always)]
pub unsafe fn send_client_request(default: Value, args: &[Value; 6]) -> Value {
    let mut value = default;
    asm!(
        "rol rdi, $$3",
        "rol rdi, $$13",
        "rol rdi, $$61",
        "rol rdi, $$51",
        "xchg rbx, rbx",
        in("rax") args.as_ptr(),
        inlateout("rdx") value,
    );
    value
}
