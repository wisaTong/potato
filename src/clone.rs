use libc;

extern "C" fn clone_cb<F>(data: *mut libc::c_void) -> libc::c_int
where
    F: FnOnce() -> isize,
{
    let boxed_fn = unsafe { Box::from_raw(data as *mut F) };
    (*boxed_fn)() as libc::c_int
}

pub unsafe fn clone<F>(f: F, stack: &mut [u8], flags: libc::c_int) -> libc::c_int
where
    F: FnOnce() -> isize,
{
    let boxed_fn = Box::new(f);
    let fn_ptr = Box::into_raw(boxed_fn) as *mut libc::c_void;

    let stack_ptr = stack.as_mut_ptr().add(stack.len());
    let stack_ptr_aligned = stack_ptr.sub(stack_ptr as usize % 16);

    libc::clone(
        clone_cb::<F>,
        stack_ptr_aligned as *mut libc::c_void,
        flags,
        fn_ptr,
    )
}
