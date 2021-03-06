use libc;
use nix::errno::{errno, Errno};

extern "C" fn clone_cb<F>(data: *mut libc::c_void) -> libc::c_int
where
    F: FnOnce() -> isize,
{
    let boxed_fn = unsafe { Box::from_raw(data as *mut F) };
    (*boxed_fn)() as libc::c_int
}

/// unsafe wrapper around libc clone
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

/// create new process to do task and create new namespaces specified in flags
pub fn clone_proc_newns<F>(f: F, stack: &mut [u8], flags: libc::c_int) -> Result<libc::c_int, Errno>
where
    F: FnOnce() -> isize,
{
    // TODO decide on more appropriate mask
    let mask = !(libc::CLONE_VM | libc::CLONE_THREAD);
    match unsafe { clone(f, stack, flags & mask) } {
        -1 => Err(Errno::from_i32(errno())),
        pid => Ok(pid),
    }
}

/// create new thread to do task and create new namespaces specified in flags,
/// silently ignore CLONE_NEWUSER and CLONE_NEWPID due to compatibility with CLONE_VM
pub fn clone_thread_newns<F>(
    f: F,
    stack: &mut [u8],
    flags: libc::c_int,
) -> Result<libc::c_int, Errno>
where
    F: FnOnce() -> isize,
{
    // TODO decide on more appropriate mask
    let mandatory = libc::CLONE_VM | libc::CLONE_THREAD | libc::CLONE_SIGHAND;
    let mask = !(libc::CLONE_NEWUSER | libc::CLONE_NEWPID);
    match unsafe { clone(f, stack, (flags | mandatory) & mask) } {
        -1 => Err(Errno::from_i32(errno())),
        pid => Ok(pid),
    }
}
