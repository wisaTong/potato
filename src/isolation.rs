use crate::{request::PotatoRequest, server::PotatoRequestHandler};
use libpotato::{clone, libc, nix, signal, signal_hook as sighook};
use nix::mount::{mount, MsFlags};
use nix::unistd;
use std::collections::HashMap;
use std::fs;
use std::io::prelude::*;
use std::net::TcpStream;
use std::os::unix::io::AsRawFd;
use std::path::Path;

#[derive(Clone)]
pub struct IsolationSetting {
    pub rootfs_path: String,
    pub mount_points: HashMap<String, String>,
}

impl IsolationSetting {
    pub fn new() -> IsolationSetting {
        IsolationSetting {
            rootfs_path: "".to_string(),
            mount_points: HashMap::new(),
        }
    }

    pub fn add_bind_mount_point(mut self, src: &str, target: &str) -> IsolationSetting {
        assert!(
            Path::new(src).exists(),
            format!("[{}] bind mount source not exist", src)
        );
        self.mount_points
            .insert(src.to_string(), target.to_string());
        self
    }

    /// Bind mounts all `src` to `target` in `mount_points`.
    /// This will silenly refuse to mount if src is not a path to a directory
    pub fn mount_all(self) -> Vec<String> {
        let mut mounted = Vec::new();
        for (src, target) in self.mount_points {
            let mnt_target = format!("{}/{}", self.rootfs_path, target);
            if Path::new(&src).is_dir() {
                fs::create_dir_all(&mnt_target).unwrap();
                if let Ok(_) = mount(
                    Some(src.as_str()),
                    mnt_target.as_str(),
                    None::<&str>,
                    MsFlags::MS_BIND,
                    None::<&str>,
                ) {
                    mounted.push(mnt_target)
                }
            }
        }
        mounted
    }
}

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
    mut stream: TcpStream,
    req: PotatoRequest,
    handler: PotatoRequestHandler,
    isolation_setting: IsolationSetting,
) -> Result<(), TcpStream> {
    let chrootfs = isolation_setting.rootfs_path.clone();
    let cleanup_fs = isolation_setting.rootfs_path.clone();
    const STACK_SIZE: usize = 1024 * 1024;

    // important: hold a fd value to close dangle connection
    let fd = stream.as_raw_fd();
    let parent_copy = stream.try_clone().unwrap();

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
            signal::default_sigcont().unwrap();
            signal::sigsuspend();

            /* whenever received SIGCONT */
            unistd::chroot(chrootfs.as_str()).unwrap();
            unistd::chdir("/").unwrap();

            let pres = handler(req);
            stream.write(&pres.to_http_response()).unwrap();
            stream.flush().unwrap();
            0 // exit
        };
        signal::block(&[nix::sys::signal::SIGCONT]);
        match clone::clone_proc_newns(worker, worker_stack, libc::SIGCHLD) {
            Ok(pid) => {
                isolation_setting.mount_all();
                proxy_signal(pid, &cleanup_fs);
            }
            Err(_) => {}
        };

        0 // exit
    };

    // mask SIGCONT of calling thread
    signal::block(&[nix::sys::signal::SIGUSR1]);
    match clone::clone_proc_newns(init, init_stack, flags) {
        Ok(pid) => {
            /* TODO: do stuff with pid (fs_prep?, idmap, gidmap, net) */

            /* Fnished set up send SIGCONT */
            unsafe { libc::kill(pid, libc::SIGUSR1) };
            unsafe { libc::close(fd) }; // close dangle connection
            Ok(())
        }
        Err(_) => Err(parent_copy),
    }
}

fn proxy_signal(pid: i32, rootfs: &str) {
    signal::set_sa_nocldstop().expect("Failed installing SIGCHLD handler");
    let sigs = [libc::SIGUSR1, libc::SIGCHLD];
    let mut siginfo = sighook::iterator::Signals::new(&sigs).unwrap(); // safe unwrap

    signal::unblock(&[nix::sys::signal::SIGUSR1]);
    for sig in siginfo.forever() {
        match sig {
            libc::SIGCHLD => {
                // fs::remove_dir_all(rootfs).unwrap();
                unsafe { libc::exit(0) }
            }
            libc::SIGUSR1 => unsafe { libc::kill(pid, libc::SIGCONT) },
            _ => unreachable!(),
        };
    }
}
