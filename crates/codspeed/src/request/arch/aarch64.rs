use std::arch::asm;

pub type Value = u64;

#[inline(always)]
pub unsafe fn send_client_request(default: Value, args: &[Value; 6]) -> Value {
    let mut value = default;
    asm!(
        "ror x12, x12, #3",
        "ror x12, x12, #13",
        "ror x12, x12, #51",
        "ror x12, x12, #61",
        "orr x10, x10, x10",
        in("x4") args.as_ptr(),
        inlateout("x3") value,
    );
    value
}
