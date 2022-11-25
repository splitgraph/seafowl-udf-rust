use serde::Serialize;

use crate::rmps::Serializer;
use anyhow::Result;
use rmpv::Value;
use std::mem;
use std::os::raw::c_void;

const SIZE_NUM_BYTES: usize = std::mem::size_of::<i32>();

fn encode(v: &Value, buf: &mut Vec<u8>) -> Result<(), rmps::encode::Error> {
    return v.serialize(&mut Serializer::new(buf));
}

fn decode(buf: &mut Vec<u8>) -> Result<Value, rmps::decode::Error> {
    return rmps::from_slice(buf);
}

#[derive(Debug)]
pub struct Decimal {
    pub precision: u8,
    pub scale: u8,
    pub value: i128,
}

#[no_mangle]
pub unsafe fn alloc(size: usize) -> *mut c_void {
    let mut buffer: Vec<u8> = Vec::with_capacity(size);
    let pointer = buffer.as_mut_ptr();
    mem::forget(buffer);

    pointer as *mut c_void
}

#[no_mangle]
pub unsafe fn dealloc(pointer: *mut c_void, capacity: usize) {
    unsafe {
        let _ = Vec::from_raw_parts(pointer, 0, capacity);
    }
}

pub unsafe fn read_input(ptr: *mut u8) -> Result<Vec<Value>> {
    let size_buf: Vec<u8> = Vec::<u8>::from_raw_parts(ptr, SIZE_NUM_BYTES, SIZE_NUM_BYTES);
    let size_bytes: [u8; SIZE_NUM_BYTES] = size_buf
        .as_slice()
        .try_into()
        .map_err(|e| anyhow::anyhow!(format!("error reading input buffer size {:?}", e)))?;
    let input_size: usize = i32::from_ne_bytes(size_bytes)
        .try_into()
        .map_err(|e| anyhow::anyhow!(format!("error converting input buffer size {:?}", e)))?;
    let mut input_buf = Vec::from_raw_parts(
        ptr.offset(SIZE_NUM_BYTES.try_into().map_err(|e| {
            anyhow::anyhow!(format!("error converting intput buffer size {:?}", e))
        })?),
        input_size,
        input_size,
    );

    let input_value = decode(&mut input_buf)?;
    let input_array = input_value
        .as_array()
        .ok_or(anyhow::anyhow!(format!(
            "error reading input buffer as array, found instead: {:?}",
            input_value
        )))?
        .to_owned();
    // Avoid deallocating the input buffer, as the host will deallocate these
    // after the call.
    mem::forget(size_buf);
    mem::forget(input_buf);
    Ok(input_array)
}

pub unsafe fn write_output(val: &Value) -> Result<*mut u8> {
    let mut serialized_output: Vec<u8> = vec![];
    // prepend serialized_output with 0 for size (to be updated after serialization)
    for _n in 0..SIZE_NUM_BYTES {
        serialized_output.push(0);
    }
    encode(val, &mut serialized_output)?;
    let output_size = serialized_output.len() - SIZE_NUM_BYTES;
    let output_size_i32: i32 = output_size
        .try_into()
        .map_err(|e| anyhow::anyhow!(format!("error converting output buffer size {:?}", e)))?;
    // write output size to beginning of output buffer
    output_size_i32
        .to_ne_bytes()
        .iter()
        .enumerate()
        .for_each(|(ix, byte)| {
            serialized_output[ix] = *byte;
        });
    let ptr = serialized_output.as_mut_ptr();
    // Output buffer will be deallocated by host
    mem::forget(serialized_output);
    Ok(ptr)
}

pub unsafe fn wrap_udf(
    input_ptr: *mut u8,
    f: impl Fn(Vec<Value>) -> Result<Value>,
) -> Result<*mut u8> {
    read_input(input_ptr)
        .and_then(|input| f(input))
        .and_then(|output| write_output(&Value::from(output)))
        .map_err(|e| {
            eprintln!("ERROR: {:?}", e);
            e
        })
}

#[allow(dead_code)]
pub fn decode_i64(v: &Value) -> Result<i64> {
    v.as_i64().ok_or(anyhow::anyhow!(format!(
        "Expected to find i64 value, but received {:?} instead",
        v
    )))
}

#[allow(dead_code)]
pub fn decode_f64(v: &Value) -> Result<f64> {
    v.as_f64().ok_or(anyhow::anyhow!(format!(
        "Expected to find f64 value, but received {:?} instead",
        v
    )))
}

#[allow(dead_code)]
pub fn decode_f32(v: &Value) -> Result<f32> {
    match v {
        Value::F32(n) => Ok(*n),
        _ => Err(anyhow::anyhow!(format!(
            "Expected to find f32 value, but received {:?} instead",
            v
        ))),
    }
}

#[allow(dead_code)]
pub fn decode_str(v: &Value) -> Result<&str> {
    v.as_str().ok_or(anyhow::anyhow!(format!(
        "Expected to find str value, but received {:?} instead",
        v
    )))
}

#[allow(dead_code)]
pub fn encode_decimal(decimal: &Decimal) -> Result<Value> {
    let low: i64 = decimal.value as i64;
    let high: i64 = (decimal.value >> 64) as i64;
    let decimal_array = Value::Array(vec![
        Value::from(decimal.precision),
        Value::from(decimal.scale),
        Value::from(high),
        Value::from(low),
    ]);
    Ok(Value::from(decimal_array))
}

#[allow(dead_code)]
pub fn decode_decimal(v: &Value) -> Result<Decimal> {
    v.as_array()
        .ok_or(anyhow::anyhow!(format!(
            "Expected to find array containing decimal parts, received {:?} instead",
            v
        )))
        .and_then(|decimal_array| {
            if decimal_array.len() != 4 {
                return Err(anyhow::anyhow!(format!(
                    "DECIMAL UDF result array should have 4 elements, found {:?} instead.",
                    decimal_array.len()
                )));
            }
            let precision = decimal_array[0]
                .as_u64()
                .ok_or(anyhow::anyhow!(format!(
                    "Decimal precision expected to be integer, found {:?} instead",
                    decimal_array[0]
                )))
                .and_then(|p_u64| {
                    let p_u8: u8 = p_u64.try_into().map_err(|err| {
                        anyhow::anyhow!(format!(
                            "Couldn't convert 64-bit precision value {:?} to u8 {:?}",
                            p_u64, err
                        ))
                    })?;
                    Ok(p_u8)
                })?;
            let scale = decimal_array[1]
                .as_u64()
                .ok_or(anyhow::anyhow!(format!(
                    "Decimal scale expected to be integer, found {:?} instead",
                    decimal_array[1]
                )))
                .and_then(|s_u64| {
                    let s_u8: u8 = s_u64.try_into().map_err(|err| {
                        anyhow::anyhow!(format!(
                            "Couldn't convert 64-bit scale value {:?} to u8 {:?}",
                            s_u64, err
                        ))
                    })?;
                    Ok(s_u8)
                })?;
            let high = decimal_array[2].as_i64().ok_or(anyhow::anyhow!(format!(
                "Decimal value high half expected to be integer, found {:?} instead",
                decimal_array[2]
            )))?;
            let low = decimal_array[3].as_i64().ok_or(anyhow::anyhow!(format!(
                "Decimal value low half expected to be integer, found {:?} instead",
                decimal_array[3]
            )))?;
            let value: i128 = (low as i128) + ((high as i128) << 64);
            Ok(Decimal {
                precision,
                scale,
                value,
            })
        })
}

#[allow(dead_code)]
pub fn decode_bool(v: &Value) -> Result<bool> {
    v.as_bool().ok_or(anyhow::anyhow!(format!(
        "Expected to find bool value, but received {:?} instead",
        v
    )))
}

#[allow(dead_code)]
pub fn decode_i32(v: &Value) -> Result<i32> {
    v.as_i64()
        .ok_or(anyhow::anyhow!(format!(
            "Expected to find i64 value, but received {:?} instead",
            v
        )))
        .and_then(|v_i64| {
            i32::try_from(v_i64)
                .map_err(|e| anyhow::anyhow!(format!("Error converting i64 to i32: {:?}", e)))
        })
}

#[allow(dead_code)]
pub fn decode_i16(v: &Value) -> Result<i16> {
    v.as_i64()
        .ok_or(anyhow::anyhow!(format!(
            "Expected to find i64 value, but received {:?} instead",
            v
        )))
        .and_then(|v_i64| {
            i16::try_from(v_i64)
                .map_err(|e| anyhow::anyhow!(format!("Error converting i64 to i16: {:?}", e)))
        })
}
