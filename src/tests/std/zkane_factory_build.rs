use hex_lit::hex;
#[allow(long_running_const_eval)]
pub fn get_bytes() -> Vec<u8> { (&hex!("0061736d0100000001070160027f7f017f030201000707010373756d00000a09010700200020016a0b")).to_vec() }