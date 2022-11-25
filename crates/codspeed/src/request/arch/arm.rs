use std::arch::asm;

pub type Value = u32;

#[inline(always)]
pub unsafe fn send_client_request(default: Value, args: &[Value; 6]) -> Value {
    let mut value = default;
    asm!(
        "mov r12, r12, ror #3",
        "mov r12, r12, ror #13",
        "mov r12, r12, ror #29",
        "mov r12, r12, ror #19",
        "orr r10, r10, r10",
        in("r4") args.as_ptr(),
        inlateout("r3") value,
    );
    value
}
