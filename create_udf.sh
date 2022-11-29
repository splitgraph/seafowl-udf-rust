#! /usr/bin/env bash
# from: https://dev.to/meleu/how-to-join-array-elements-in-a-bash-script-303a#join-elements-with-a-string
joinByString() {
  local separator="$1"
  shift
  local first="$1"
  shift
  printf "%s" "$first" "${@/#/$separator}"
}

# Parameters to set
filename="target/wasm32-wasi/release/seafowl_udf_rust.wasm"
function_name="add_i643"
wasm_export="add_i64"
return_type="BIGINT"
input_types=("BIGINT" "BIGINT")

cat << EndOfMessage
CREATE FUNCTION $function_name AS '
{
  "entrypoint": "$wasm_export",
  "language": "wasmMessagePack",
  "input_types": ["$(joinByString '", "' "${input_types[@]}")"],
  "return_type": "$return_type",
  "data": "$(base64 -i $filename)"
}';
EndOfMessage

