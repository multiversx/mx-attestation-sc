#![no_main]
use libfuzzer_sys::fuzz_target;

use elrond_wasm::elrond_codec::*;
use elrond_wasm_debug::*;

use attestation::user::User;
use elrond_codec::top_de::TopDecode;

fuzz_target!(|data: &[u8]| {
    if let Ok(decoded) = User::top_decode(&mut &data[..]) {
        let encoded_clean = decoded.top_encode().unwrap();
        let decoded_again = User::top_decode(&mut &encoded_clean[..]).unwrap();
        // assert_eq!(decoded, decoded_again);
        let encoded_again = decoded_again.top_encode().unwrap();
        assert_eq!(encoded_clean, encoded_again);
    }
});
