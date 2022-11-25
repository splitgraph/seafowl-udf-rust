const fs = require('fs');
const b64encode = (buf) => buf.toString('base64');

const readBuffer = filename => fs.readFileSync(filename);

const get_sql = ({udf_name, input_types, return_type, wasm_export_name, b64wasm}) => `
CREATE FUNCTION ${udf_name} AS '
{
  "entrypoint": "${wasm_export_name ?? udf_name}",
  "language": "wasmMessagePack",
  "input_types": ["${input_types.join('", "')}"],
  "return_type": "${return_type}",
  "data": "${b64wasm}"
}';
`

const main = (argv) => {
  const input_types = ["BIGINT", "BIGINT"];
  const return_type = "BIGINT";
  const [udf_name, wasm_export_name, wasm_filename] = argv;
  const b64wasm = b64encode(readBuffer(wasm_filename));
  console.log(get_sql({udf_name, input_types, return_type, wasm_export_name, b64wasm}));
}


if (require.main === module) {
  main(process.argv.slice(2));
}
