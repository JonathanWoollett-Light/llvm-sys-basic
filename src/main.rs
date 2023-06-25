use llvm_sys::analysis::LLVMVerifierFailureAction::LLVMAbortProcessAction;
use llvm_sys::core::LLVMBuildRet;
use llvm_sys::execution_engine::{LLVMGetFunctionAddress, LLVMLinkInInterpreter};
use llvm_sys::target::{LLVM_InitializeNativeAsmParser, LLVM_InitializeNativeAsmPrinter};
use llvm_sys::{
    analysis::LLVMVerifyModule,
    core::{
        LLVMAddFunction, LLVMAppendBasicBlock, LLVMBuildAdd, LLVMCreateBuilder, LLVMDisposeMessage,
        LLVMFunctionType, LLVMGetParam, LLVMInt32Type, LLVMModuleCreateWithName,
        LLVMPositionBuilderAtEnd,
    },
    execution_engine::{
        LLVMCreateExecutionEngineForModule, LLVMCreateGenericValueOfInt, LLVMExecutionEngineRef,
        LLVMGenericValueToInt, LLVMLinkInMCJIT, LLVMRunFunction,
    },
    target::LLVM_InitializeNativeTarget,
};

use std::ffi::{CStr, CString};

// From https://www.pauladamsmith.com/blog/2015/01/how-to-get-started-with-llvm-c-api.html

fn main() {
    unsafe {
        let module_name = CString::new("my_module").unwrap();
        let module = LLVMModuleCreateWithName(module_name.as_c_str().as_ptr());

        let mut param_types = [LLVMInt32Type(), LLVMInt32Type()];
        let ret_type = LLVMFunctionType(LLVMInt32Type(), param_types.as_mut_ptr(), 2, 0);

        let function_name = CString::new("sum").unwrap();
        let sum = LLVMAddFunction(module, function_name.as_c_str().as_ptr(), ret_type);
        let block_name = CString::new("entry").unwrap();
        let entry = LLVMAppendBasicBlock(sum, block_name.as_c_str().as_ptr());

        let builder = LLVMCreateBuilder();
        LLVMPositionBuilderAtEnd(builder, entry);

        let result_name = CString::new("temp").unwrap();
        let temp = LLVMBuildAdd(
            builder,
            LLVMGetParam(sum, 0),
            LLVMGetParam(sum, 1),
            result_name.as_c_str().as_ptr(),
        );
        LLVMBuildRet(builder, temp);

        // Verify the module
        let mut error = std::ptr::null_mut();
        LLVMVerifyModule(module, LLVMAbortProcessAction, &mut error);
        LLVMDisposeMessage(error);

        // Run the module using a JIT execution engine
        let mut engine = std::ptr::null_mut();
        error = std::ptr::null_mut();

        LLVMLinkInMCJIT();
        // LLVMLinkInInterpreter();
        LLVM_InitializeNativeTarget();

        if LLVMCreateExecutionEngineForModule(&mut engine, module, &mut error) != 0 {
            println!("failed to create execution engine");
            return;
        }

        if !error.is_null() {
            println!("error: {}", CStr::from_ptr(error).to_str().unwrap());
            LLVMDisposeMessage(error);
            return;
        }

        // This was previously needed with `LLVMRunFunction`.
        // let mut args = [
        //     LLVMCreateGenericValueOfInt(LLVMInt32Type(), 1, 0),
        //     LLVMCreateGenericValueOfInt(LLVMInt32Type(), 2, 0),
        // ];

        // I don't know why these are needed, although they appear to resolve 1 of the issues.
        // From https://stackoverflow.com/a/38801376/4301453
        LLVM_InitializeNativeAsmPrinter();
        LLVM_InitializeNativeAsmParser();

        // I was experiencing issues with `LLVMRunFunction` looking this up I found
        // https://stackoverflow.com/a/63440756/4301453 which says to use `LLVMGetFunctionAddress`.
        let addr = LLVMGetFunctionAddress(engine, function_name.as_c_str().as_ptr());
        let ptr = addr as *mut fn(i32, i32) -> i32;
        println!("got here");
        let res = (*ptr)(1i32, 2i32);
        println!("didn't get here");
        println!("{}", res);

        // let res = LLVMRunFunction(engine, sum, 2, args.as_mut_ptr());
        // println!("{}", LLVMGenericValueToInt(res, 0));
    }
}
