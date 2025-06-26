use hex_lit::hex;
#[allow(long_running_const_eval)]
pub fn get_bytes() -> Vec<u8> { (&hex!("0061736d01000000010401600000030201000a040102000b")).to_vec() }