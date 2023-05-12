/*
{
  "type": "invoke",
  "request_id": "63035d58-4840-446f-833a-e09d88e07aa1",
  "name": "echo3",
  "uri": "assets/echo3_3724494060.wasm",
  "signature": {
    "params": [],
    "shim_idx": 0,
    "ret": {
      "type": "vector",
      "content": {
        "type": "u8"
      }
    },
    "inner_ret": {
      "type": "vector",
      "content": {
        "type": "u8"
      }
    }
  },
  "args": []
}
 */

const imports = {
  __wbindgen_placeholder__: {
    __wbindgen_describe: () => {},
  },
  __wbindgen_externref_xform__: {
    __wbindgen_externref_table_grow: () => {
      throw Error("Unexpected call to __wbindgen_externref_table_grow");
    },
    __wbindgen_externref_table_set_null: () => {
      throw Error("Unexpected call to __wbindgen_externref_table_set_null");
    },
  },
};
const maybeReturnPtr = (wasmInstance, returnSignature) => {
  let type = returnSignature.type;
  switch(type) {
    case "string":
    case "vector":
    case "ref":
    case "refmut":
    case "slice":
      {
        let returnPtr = wasmInstance.exports.__wbindgen_malloc(16);
        return returnPtr;
      }
    default:
      return null;
  }
}

const vectorLookup = {
  "u8": [Uint8Array, 1],
  "i8": [Int8Array, 1],
  "u16": [Uint16Array, 2],
  "i16": [Int16Array, 2],
  "u32": [Uint32Array, 4],
  "i32": [Int32Array, 4],
  "f32": [Float32Array, 4],
  "f64": [Float64Array, 8],
};

const parseParam = (wasmInstance, param, arg) => {
  const malloc = wasmInstance.exports.__wbindgen_malloc;
  const memory = wasmInstance.exports.memory;
  switch(param) {
    case "u8":
    case "i8":
    case "u16":
    case "i16":
    case "u32":
    case "i32":
    case "f32":
    case "f64":
      return [arg];
    case "i64":
    case "u64":
      return [BigInt(arg)];
    case "string":
      {
        const ptr = malloc(arg.length);
        var memoryView = new Uint8Array(memory.buffer);
        let encoder = new TextEncoder('utf-8');
        let argBytes = encoder.encode(arg);
        memoryView = memoryView.subarray(ptr, ptr + argBytes.length);
        memoryView.set(argBytes);
        return [ptr, argBytes.length];
      }
    case "ref":
    case "refmut":
      {
        let subType = param.content;
        return parseParam(wasmInstance, subType, arg);
      }
    case "vector":
    case "slice":
      {
        let subType = param.content;
        let [VectorType, sizeModifier] = vectorLookup[subType.type];
        if (!VectorType)
          throw Error("Unsupported type List of %s", subType.type);
        let byteLength = arg.length * sizeModifier;
        const ptr = malloc(byteLength);
        const ptrAdjusted = ptr / sizeModifier;
        var memoryView = new VectorType(memory.buffer);
        memoryView = memoryView.subArray(ptrAdjusted, ptrAdjusted + arg.length);
        memoryView.set(arg);
        return [ptr, arg.length];
      }
  }
}

const parseReturnPtr = (wasmInstance, returnSignature, returnPtr) => {
  let type = returnSignature.type;
  let memory = wasmInstance.exports.memory;
  let memoryView32 = new Int32Array(memory.buffer);
  let subPtr = memoryView32[returnPtr / 4];
  let ptrLength = memoryView32[(returnPtr / 4) + 1];

  switch(type) {
    case "ref":
    case "refmut":
      {
        let subType = returnSignature.content;
        return parseReturnPtr(wasmInstance, subType, subPtr);
      }
    case "string":
      {
        var memoryView = new Uint8Array(memory.buffer);
        memoryView = memoryView.subarray(subPtr, subPtr + ptrLength);
        let decoder = new TextDecoder("utf-8");
        return decoder.decode(memoryView);
      }
    case "vector":
    case "slice":
      {
        let subType = returnSignature.content;
        let [VectorType, sizeModifier] = vectorLookup[subType.type];
        if (!VectorType)
          throw Error("Failed to parse return vector of type: %s", JSON.stringify(returnSignature));
        memoryView = new VectorType(memory.buffer);
        let ptrAdjusted = subPtr / sizeModifier;
        memoryView = memoryView.subarray(ptrAdjusted, ptrAdjusted + ptrLength);
        return Array.from(memoryView);
      }
  }
}

module.exports.execute = async (wasmBytes, fn_name, signature, args) => {
  let wasmModule = await WebAssembly.instantiate(wasmBytes, imports);
  let wasmInstance = wasmModule.instance;
  let wasmArgs = [];
  if (signature.params.length !== args.length)
    throw Error("Wrong number of arguments, expected %d", signature.params.length);

  let fn = wasmInstance.exports[fn_name];
  if (!fn)
    throw Error("No such function: %s", fn_name);

  let returnPtr = maybeReturnPtr(wasmInstance, signature.ret);
  if (returnPtr !== null)
    wasmArgs.push(returnPtr);

  signature.params.map( (param, index) => {
    let parsedArgs = parseParam(wasmInstance, param, args[index]);
    wasmArgs.concat(parsedArgs);
  });
  let returnedVal = fn(...wasmArgs);

  if (!returnPtr) {
    return returnedVal;
  } else {
    return parseReturnPtr(wasmInstance, signature.ret, returnPtr);
  }
}
