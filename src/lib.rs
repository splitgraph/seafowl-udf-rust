extern crate rmp_serde as rmps;
extern crate serde;
use rmpv::Value;
use udf::{decode_i64};

mod udf;

fn do_add(left: i64, right: i64) -> i64 {
    left + right
}

#[no_mangle]
pub unsafe fn add_i64(input_ptr: *mut u8) -> *mut u8 {
    udf::wrap_udf(input_ptr, |args| {
        Ok(Value::from(do_add(
            decode_i64(&args[0])?,
            decode_i64(&args[1])?
        )))
    })
    .unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add() {
        let result = do_add(2_i64, 3_i64);
        assert_eq!(result, 5_i64);
    }
}
