use std::ffi::c_void;
use libffi_sys as ffi;

pub struct Symbol {
    cif: ffi::ffi_cif,
    ptr: *mut c_void,
}

pub fn to_ffi_type<'s>(scope: &mut v8::HandleScope<'s>, value: v8::Local<v8::Value>) -> ffi::ffi_type {
    unsafe {
        if value.is_string() {
            let string = value.to_rust_string_lossy(scope);
            match string.as_str() {
                "u8" => ffi::ffi_type_uint8,
                "i8" => ffi::ffi_type_sint8,
                "u16" => ffi::ffi_type_uint16,
                "i16" => ffi::ffi_type_sint16,
                "i32" => ffi::ffi_type_sint32,
                "u32" => ffi::ffi_type_uint32,
                "i64" => ffi::ffi_type_sint64,
                "u64" => ffi::ffi_type_uint64,
                "f32" => ffi::ffi_type_float,
                "f64" => ffi::ffi_type_double,
                "pointer" => ffi::ffi_type_pointer,
                "void" => ffi::ffi_type_void,
                _ => panic!("Unknown type: {}", string),
            }
        } else {
            ffi::ffi_type_void
        }
    }
}

pub fn ffi_function<'s>(
    scope: &mut v8::HandleScope<'s>,
    name: &str,
    ptr: *mut c_void,
    def: v8::Local<v8::Object>,
) -> v8::Local<'s, v8::Function> {
    let mut cif = ffi::ffi_cif::default();

    let rtype_name = v8::String::new(scope, "result").unwrap();
    let rtype_val = def.get(scope, rtype_name.into()).unwrap();
    let rtype = Box::new(to_ffi_type(scope, rtype_val));

    let atype_name = v8::String::new(scope, "parameters").unwrap();
    let atype_val = v8::Local::<v8::Array>::try_from(def.get(scope, atype_name.into()).unwrap()).unwrap();
    
    let mut parameters: Box<Vec<*mut ffi::ffi_type>> = Box::new(vec![std::ptr::null_mut(); atype_val.length() as usize]);
    
    for idx in 0..atype_val.length() {
        let atype_val = atype_val.get_index(scope, idx).unwrap();
        let atype = to_ffi_type(scope, atype_val);
        parameters[idx as usize] = Box::leak(Box::new(atype)) as *mut _;
    }

    unsafe {
        ffi::ffi_prep_cif(
            &mut cif as *mut _,
            ffi::ffi_abi_FFI_DEFAULT_ABI,
            parameters.len() as u32,
            Box::leak(rtype) as *mut _,
            parameters.as_mut_ptr()
        );
    }

    let _ = Box::leak(parameters);

    let external = v8::External::new(scope, Box::into_raw(Box::new(Symbol { cif, ptr })) as *mut _);

    let func = v8::Function::builder(
        |scope: &mut v8::HandleScope,
            args: v8::FunctionCallbackArguments,
            mut rv: v8::ReturnValue|
        {
            let ptr = v8::Local::<v8::External>::try_from(args.data().unwrap()).unwrap().value() as *mut Symbol;
            let mut symbol = unsafe { Box::from_raw(ptr) };

            let mut avalue = vec![std::ptr::null_mut::<c_void>(); symbol.cif.nargs as usize];
            let atypes = unsafe { std::slice::from_raw_parts(symbol.cif.arg_types, symbol.cif.nargs as usize) };
            
            for i in 0..symbol.cif.nargs { 
                let atype = unsafe { &*(atypes[i as usize]) };
                match atype.type_ as u32 {
                    ffi::FFI_TYPE_UINT8 => {
                        let arg = args.get(i as i32);
                        avalue[i as usize] = Box::into_raw(Box::new(arg.uint32_value(scope).unwrap() as u8)) as *mut c_void;
                    }
                    ffi::FFI_TYPE_SINT8 => {
                        let arg = args.get(i as i32);
                        avalue[i as usize] = Box::into_raw(Box::new(arg.int32_value(scope).unwrap() as i8)) as *mut c_void;
                    }
                    ffi::FFI_TYPE_UINT16 => {
                        let arg = args.get(i as i32);
                        avalue[i as usize] = Box::into_raw(Box::new(arg.uint32_value(scope).unwrap() as u16)) as *mut c_void;
                    }
                    ffi::FFI_TYPE_SINT16 => {
                        let arg = args.get(i as i32);
                        avalue[i as usize] = Box::into_raw(Box::new(arg.int32_value(scope).unwrap() as i16)) as *mut c_void;
                    }
                    ffi::FFI_TYPE_UINT32 => {
                        let arg = args.get(i as i32);
                        avalue[i as usize] = Box::into_raw(Box::new(arg.uint32_value(scope).unwrap())) as *mut c_void;
                    }
                    ffi::FFI_TYPE_SINT32 => {
                        let arg = args.get(i as i32);
                        avalue[i as usize] = Box::into_raw(Box::new(arg.int32_value(scope).unwrap())) as *mut c_void;
                    }
                    ffi::FFI_TYPE_UINT64 => {
                        let arg = args.get(i as i32);
                        if arg.is_big_int() {
                            let bigint = v8::Local::<v8::BigInt>::try_from(arg).unwrap();
                            avalue[i as usize] = Box::into_raw(Box::new(bigint.u64_value())) as *mut c_void;
                        } else {
                            avalue[i as usize] = Box::into_raw(Box::new(arg.uint32_value(scope).unwrap() as u64)) as *mut c_void;
                        }
                    }
                    ffi::FFI_TYPE_SINT64 => {
                        let arg = args.get(i as i32);
                        if arg.is_big_int() {
                            let bigint = v8::Local::<v8::BigInt>::try_from(arg).unwrap();
                            avalue[i as usize] = Box::into_raw(Box::new(bigint.i64_value())) as *mut c_void;
                        } else {
                            avalue[i as usize] = Box::into_raw(Box::new(arg.int32_value(scope).unwrap() as i64)) as *mut c_void;
                        }
                    }
                    ffi::FFI_TYPE_FLOAT => {
                        let arg = args.get(i as i32);
                        avalue[i as usize] = Box::into_raw(Box::new(arg.number_value(scope).unwrap() as f32)) as *mut c_void;
                    }
                    ffi::FFI_TYPE_DOUBLE => {
                        let arg = args.get(i as i32);
                        avalue[i as usize] = Box::into_raw(Box::new(arg.number_value(scope).unwrap() as f64)) as *mut c_void;
                    }
                    ffi::FFI_TYPE_POINTER => {
                        let arg = args.get(i as i32);
                        if arg.is_big_int() {
                            let bigint = v8::Local::<v8::BigInt>::try_from(arg).unwrap();
                            avalue[i as usize] = Box::into_raw(Box::new(bigint.u64_value())) as *mut c_void;
                        } else if arg.is_typed_array() {
                            let typed_array = v8::Local::<v8::TypedArray>::try_from(arg).unwrap();
                            let buffer = typed_array.buffer(scope).unwrap();
                            let store = buffer.get_backing_store();
                            let offset = typed_array.byte_offset();
                            avalue[i as usize] = Box::into_raw(Box::new(store.data().unwrap().as_ptr() as u64 + offset as u64)) as *mut c_void;
                        } else if arg.is_array_buffer() {
                            let buffer = v8::Local::<v8::ArrayBuffer>::try_from(arg).unwrap();
                            let store = buffer.get_backing_store();
                            avalue[i as usize] = Box::into_raw(Box::new(store.data().unwrap().as_ptr() as u64)) as *mut c_void;
                        } else if arg.is_null() {
                            avalue[i as usize] = Box::into_raw(Box::new(0u64)) as *mut c_void;
                        } else {
                            unimplemented!()
                        }
                    }
                    _ => unimplemented!()
                }
            }
            
            let rtype = unsafe { &*symbol.cif.rtype };
            let mut rvalue = vec![0u8; rtype.size];

            unsafe {
                ffi::ffi_call(
                    &mut symbol.cif as *mut _,
                    Some(std::mem::transmute(symbol.ptr)),
                    rvalue.as_mut_ptr() as *mut _,
                    avalue.as_mut_ptr() as *mut _,
                );
            }
            let _ = Box::leak(symbol);
            let result: v8::Local<v8::Value> = unsafe {match rtype.type_ as u32 {
                ffi::FFI_TYPE_UINT8 => v8::Number::new(scope, rvalue[0] as f64).into(),
                ffi::FFI_TYPE_SINT8 => v8::Number::new(scope, rvalue[0] as f64).into(),
                ffi::FFI_TYPE_UINT16 => v8::Number::new(scope, (&*(rvalue.as_ptr() as *const [u16; 1]))[0] as f64).into(),
                ffi::FFI_TYPE_SINT16 => v8::Number::new(scope, (&*(rvalue.as_ptr() as *const [i16; 1]))[0] as f64).into(),
                ffi::FFI_TYPE_UINT32 => v8::Number::new(scope, (&*(rvalue.as_ptr() as *const [u32; 1]))[0] as f64).into(),
                ffi::FFI_TYPE_SINT32 => v8::Number::new(scope, (&*(rvalue.as_ptr() as *const [i32; 1]))[0] as f64).into(),
                ffi::FFI_TYPE_UINT64 => v8::BigInt::new_from_u64(scope, (&*(rvalue.as_ptr() as *const [u64; 1]))[0]).into(),
                ffi::FFI_TYPE_SINT64 => v8::BigInt::new_from_i64(scope, (&*(rvalue.as_ptr() as *const [i64; 1]))[0]).into(),
                ffi::FFI_TYPE_FLOAT => v8::Number::new(scope, (&*(rvalue.as_ptr() as *const [f32; 1]))[0] as f64).into(),
                ffi::FFI_TYPE_DOUBLE => v8::Number::new(scope, (&*(rvalue.as_ptr() as *const [f64; 1]))[0] as f64).into(),
                ffi::FFI_TYPE_POINTER => v8::BigInt::new_from_u64(scope, rvalue.as_ptr() as u64).into(),
                _ => v8::undefined(scope).into(),
            }};
            rv.set(result);
        })
        .data(external.into())
        .build(scope)
        .unwrap();

    let name = v8::String::new(scope, &format!("{} at {:?}", name, ptr)).unwrap();
    func.set_name(name);

    func
}

pub fn class_dynamic_library<'s>(scope: &mut v8::HandleScope<'s>) -> v8::Local<'s, v8::Function> {
    let ctor = v8::FunctionTemplate::builder(
        |scope: &mut v8::HandleScope,
            args: v8::FunctionCallbackArguments,
            mut _rv: v8::ReturnValue|
        {
            let this = args.this().to_object(scope).unwrap();

            let path = args.get(0).to_rust_string_lossy(scope);
            let symbols = args.get(1).to_object(scope).unwrap();
            
            let symbols_object = v8::Object::new(scope);
            
            let symbols_name = v8::String::new(scope, "symbols").unwrap();
            this.set(scope, symbols_name.into(), symbols_object.into());
            
            #[cfg(windows)]
            {
                use winapi::um::libloaderapi::LoadLibraryA;
                use winapi::um::libloaderapi::GetProcAddress;

                let pathcstr = std::ffi::CString::new(path.as_bytes()).unwrap();
                let handle = unsafe { LoadLibraryA(pathcstr.as_ptr()) };
                if handle.is_null() {
                    let message = v8::String::new(scope, &format!("Failed to load library: {}", path)).unwrap();
                    let exception = v8::Exception::error(scope, message);
                    scope.throw_exception(exception);
                    return;
                }

                let symbol_names = symbols.get_property_names(scope).unwrap();
                for i in 0..symbol_names.length() {
                    let name = symbol_names.get_index(scope, i).unwrap();
                    let name_str = name.to_rust_string_lossy(scope);
                    let name_cstr = std::ffi::CString::new(name_str.as_bytes()).unwrap();
                    let ptr = unsafe { GetProcAddress(handle, name_cstr.as_ptr()) };
                    if ptr.is_null() {
                        let message = v8::String::new(scope, &format!("Failed to find symbol: {}", name_str)).unwrap();
                        let exception = v8::Exception::error(scope, message);
                        scope.throw_exception(exception);
                        return;
                    }
                    let def = symbols.get(scope, name).unwrap().to_object(scope).unwrap();
                    let function = ffi_function(scope, &name_str, ptr as *mut _, def).into();
                    symbols_object.set(scope, name, function);
                }
            }
            #[cfg(not(windows))]
            {
                let message = v8::String::new(scope, "Not implemented").unwrap();
                let exception = v8::Exception::error(scope, message);
                scope.throw_exception(exception);
                return;
            }
        })
        .build(scope);
    
    let class_name = v8::String::new(scope, "DynamicLibrary").unwrap();
    ctor.set_class_name(class_name);

    ctor.get_function(scope).unwrap()
}
