mod ffi;

fn main() {
    let platform = v8::new_default_platform(0, false).make_shared();
    v8::V8::initialize_platform(platform);
    v8::V8::initialize();

    let isolate = &mut v8::Isolate::new(Default::default());

    let scope = &mut v8::HandleScope::new(isolate);
    let context = v8::Context::new(scope);
    let scope = &mut v8::ContextScope::new(scope, context);

    let print_fn = v8::Function::builder(
        |scope: &mut v8::HandleScope,
            args: v8::FunctionCallbackArguments,
            mut _rv: v8::ReturnValue|
        {
            let mut result = String::new();
            for idx in 0..(args.length()) {
                result.push_str(&args.get(idx).to_rust_string_lossy(scope));
            }
            println!("{}", result);
        })
        .build(scope)
        .unwrap();

    let start_time = Box::leak(Box::new(std::time::Instant::now())) as *mut std::time::Instant;
    let start_time = v8::External::new(scope, start_time as *mut _);

    let now_fn = v8::Function::builder( 
        |scope: &mut v8::HandleScope,
            args: v8::FunctionCallbackArguments,
            mut rv: v8::ReturnValue|
        {
            let start_time = v8::Local::<v8::External>::try_from(args.data().unwrap()).unwrap();
            let start_time = unsafe { &*(start_time.value() as *const std::time::Instant) };
            let seconds = start_time.elapsed().as_secs();
            let subsec_nanos = start_time.elapsed().subsec_nanos() as f64;
            let result = (seconds * 1_000) as f64 + (subsec_nanos / 1_000_000.0);
            rv.set(v8::Number::new(scope, result).into());
        })
        .data(start_time.into())
        .build(scope)
        .unwrap();
    
    let global = context.global(scope);
    let print_fn_name = v8::String::new(scope, "print").unwrap();
    global.set(
        scope,
        print_fn_name.into(),
        print_fn.into()
    );
    let now_fn_name = v8::String::new(scope, "now").unwrap();
    global.set(
        scope,
        now_fn_name.into(),
        now_fn.into()
    );

    let class_dynamic_library = ffi::class_dynamic_library(scope);
    let class_name = v8::String::new(scope, "DynamicLibrary").unwrap();
    global.set(
        scope,
        class_name.into(),
        class_dynamic_library.into()
    );

    let code = v8::String::new(scope, &std::fs::read_to_string("test.js").unwrap()).unwrap();

    let script = v8::Script::compile(scope, code, None).unwrap();
    let _result = script.run(scope).unwrap();
    // let result = result.to_string(scope).unwrap();
    // println!("result: {}", result.to_rust_string_lossy(scope));
}
