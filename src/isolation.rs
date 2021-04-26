use crate::{request::PotatoRequest, server::PotatoRequestHandler};
use libpotato::{clone, libc, nix, signal, signal_hook as sighook};
use nix::unistd;
use std::fs;
use std::io::prelude::*;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;

/// NOTE: becareful of what you handler do.
/// Your handler will be execute in new process in different namespaces
/// and might cause unexpected behavior.
/// A common case is when your handler is dealing with system resources
///
/// Thread that calls this function will be made to
/// automatically mask SIGCONT before calling `clone`
/// so that the new init process will inherit the signal masks
///
/// On failure return the ownership of TcpStream for furthur error handling
pub fn isolate_req(
    stream: TcpStream,
    req: PotatoRequest,
    handler: PotatoRequestHandler,
    rootfs: &str,
) -> Result<(), TcpStream> {
    const STACK_SIZE: usize = 1024 * 1024;

    // important: hold a fd value to close dangle connection
    // and give worker a copy of the stream
    let fd = stream.as_raw_fd();
    let mut worker_copy = stream.try_clone().unwrap();

    let flags = libc::CLONE_NEWUSER
        | libc::CLONE_NEWPID
        | libc::CLONE_NEWNS
        | libc::CLONE_NEWUTS
        | libc::CLONE_NEWIPC
        | libc::CLONE_NEWNET
        | libc::CLONE_NEWCGROUP
        | libc::SIGCHLD;

    let ref mut init_stack = [0; STACK_SIZE];
    let init = move || {
        let ref mut worker_stack = [0; STACK_SIZE];
        let worker = move || {
            /* start in stopped state */
            signal::unblock(&[nix::sys::signal::SIGCONT]);
            unsafe { libc::raise(libc::SIGSTOP) };

            /* whenever received SIGCONT */
            unistd::chroot(rootfs).unwrap();
            unistd::chdir(".").unwrap();

            let pres = handler(req);
            worker_copy.write(&pres.to_http_response()).unwrap();
            worker_copy.flush().unwrap();
            0 // exit
        };
        match clone::clone_proc_newns(worker, worker_stack, libc::SIGCHLD) {
            Ok(pid) => proxy_signal(pid, rootfs),
            Err(_) => {}
        };

        0 // exit
    };

    // mask SIGCONT of calling thread
    signal::block(&[nix::sys::signal::SIGCONT]);
    match clone::clone_proc_newns(init, init_stack, flags) {
        Ok(pid) => {
            /* TODO: do stuff with pid (fs_prep?, idmap, gidmap, net) */

            /* Fnished set up send SIGCONT */
            unsafe { libc::kill(pid, libc::SIGCONT) };
            unsafe { libc::close(fd) }; // close dangle connection
            Ok(())
        }
        Err(_) => Err(stream),
    }
}

fn proxy_signal(pid: i32, rootfs: &str) {
    signal::set_sa_nocldstop().expect("Failed installing SIGCHLD handler");
    let sigs = [libc::SIGCONT, libc::SIGCHLD];
    let mut siginfo = sighook::iterator::Signals::new(&sigs).unwrap(); // safe unwrap

    // unblock after installed handler
    signal::unblock(&[nix::sys::signal::SIGCONT]);
    for sig in siginfo.forever() {
        match sig {
            libc::SIGCHLD => {
                fs::remove_dir_all(rootfs).unwrap();
                unsafe { libc::exit(0) }
            }
            libc::SIGCONT => unsafe { libc::kill(pid, sig) },
            _ => unreachable!(),
        };
    }
}
