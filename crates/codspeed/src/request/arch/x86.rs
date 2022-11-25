use std::arch::asm;

pub type Value = u32;

#[inline(always)]
pub unsafe fn send_client_request(default: Value, args: &[Value; 6]) -> Value {
    let mut value = default;
    asm!(
        "rol edi, $$3",
        "rol edi, $$13",
        "rol edi, $$29",
        "rol edi, $$19",
        "xchg ebx, ebx",
        in("eax") args.as_ptr(),
        inlateout("edx") value,
    );
    value
}
