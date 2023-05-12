use thiserror::Error;
use wasmer::{
    imports, Function as HostFunction, FunctionEnv as HostFunctionEnv,
    FunctionEnvMut as HostFunctionEnvMut, Instance, Module, Store,
};

pub fn extract_description(bytes: &[u8]) -> Result<(String, Vec<u8>), WasmError> {
    let mut store = Store::default();
    let module = Module::new(&store, bytes).map_err(|_| WasmError::InvalidModule)?;
    fn description(mut env: HostFunctionEnvMut<Vec<u8>>, x: u8) {
        env.data_mut().push(x);
    }

    let env = HostFunctionEnv::new(&mut store, Vec::new());
    let import_obj = imports! {
        "__wbindgen_placeholder__" => {
            "__wbindgen_describe" => HostFunction::new_typed_with_env(
                &mut store,
                &env,
                description)
        },
        // NOTE: calls to these imports should not take place during
        // function signature extraction
        "__wbindgen_externref_xform__" => {
            "__wbindgen_externref_table_grow" => HostFunction::new_typed(&mut store, |_x: i32|{
                panic!("Call to __wbindgen_externref_table_grow");
                // XXX: Unreachable, but just to make Wasmer happy
                _x
            }),
            "__wbindgen_externref_table_set_null" => HostFunction::new_typed(&mut store, |_: i32|{
                panic!("Call to __wbindgen_externref_table_set_null")
            })
        }
    };

    let instance = Instance::new(&mut store, &module, &import_obj).map_err(|e| {
        tracing::warn!("Failed to load instance: {e:?}");
        WasmError::LoadingInstance
    })?;
    let mut maybe_name = None;
    let desc_prefix = "__wbindgen_describe_";
    for (export_name, _) in instance.exports.iter() {
        if export_name.starts_with(desc_prefix) && maybe_name.is_none() {
            maybe_name = Some(String::from(&export_name[desc_prefix.len()..]));
        } else if export_name.starts_with(desc_prefix) && maybe_name.is_some() {
            return Err(WasmError::MultipleExports);
        }
    }

    let name = maybe_name.ok_or(WasmError::FunctionNotFound)?;
    tracing::trace!("Detected exported function: {}", name);

    let desc_func_name = format!("__wbindgen_describe_{}", name);
    let wasm_desc_func = instance
        .exports
        .get_function(&desc_func_name)
        .map_err(|_| WasmError::FunctionNotFound)?;
    wasm_desc_func
        .call(&mut store, &vec![])
        .map_err(|_| WasmError::CorruptFunctionDesc)?;
    let result = env.as_ref(&store).clone();
    Ok((name, result))
}

#[derive(Debug, Error)]
pub enum WasmError {
    #[error("could not parse wasm instance")]
    LoadingInstance,
    #[error("corrupt wasm module")]
    InvalidModule,
    #[error("no function were found in binary")]
    FunctionNotFound,
    #[error("corrupt function signature")]
    CorruptFunctionDesc,
    #[error("multiple functions were exported, only export one function")]
    MultipleExports,
}
