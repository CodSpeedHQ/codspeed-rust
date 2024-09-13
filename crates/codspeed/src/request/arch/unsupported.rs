pub type Value = u32;

#[inline(always)]
pub unsafe fn send_client_request(_default: Value, _args: &[Value; 6]) -> Value {
    panic!("Not implemented for this architecture");
}
