use thiserror::Error;
use wasmer::{
    imports, Function as HostFunction, FunctionEnv as HostFunctionEnv,
    FunctionEnvMut as HostFunctionEnvMut, Instance, Module, Store,
};

pub fn extract_description(name: &str, bytes: &[u8]) -> Result<Vec<u8>, WasmError> {
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
    let desc_func_name = format!("__wbindgen_describe_{}", name);
    let wasm_desc_func = instance
        .exports
        .get_function(&desc_func_name)
        .map_err(|_| WasmError::FunctionNotFound(String::from(name)))?;
    wasm_desc_func
        .call(&mut store, &vec![])
        .map_err(|_| WasmError::CorruptFunctionDesc)?;
    let result = env.as_ref(&store).clone();
    Ok(result)
}

#[derive(Debug, Error)]
pub enum WasmError {
    #[error("could not parse wasm instance")]
    LoadingInstance,
    #[error("corrupt wasm module")]
    InvalidModule,
    #[error("function({0}) not found in binary")]
    FunctionNotFound(String),
    #[error("corrupt function signature")]
    CorruptFunctionDesc,
}
