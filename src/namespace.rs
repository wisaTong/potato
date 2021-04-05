use nix::sched::CloneFlags;
use std::fs::OpenOptions;
use std::io;
use std::os::unix::io::{IntoRawFd, RawFd};

#[derive(Debug)]
pub enum Namespace {
    Cgroup,
    Ipc,
    Network,
    Mount,
    Pid,
    User,
    Uts,
}

impl Namespace {
    pub fn to_symlink_name(&self) -> &str {
        match self {
            Namespace::Cgroup => "cgroup",
            Namespace::Ipc => "ipc",
            Namespace::Network => "net",
            Namespace::Mount => "mnt",
            Namespace::Pid => "pid",
            Namespace::User => "user",
            Namespace::Uts => "uts",
        }
    }

    pub fn to_clone_flag(&self) -> CloneFlags {
        match self {
            Namespace::Cgroup => CloneFlags::CLONE_NEWCGROUP,
            Namespace::Ipc => CloneFlags::CLONE_NEWIPC,
            Namespace::Network => CloneFlags::CLONE_NEWNET,
            Namespace::Mount => CloneFlags::CLONE_NEWNS,
            Namespace::Pid => CloneFlags::CLONE_NEWPID,
            Namespace::User => CloneFlags::CLONE_NEWUSER,
            Namespace::Uts => CloneFlags::CLONE_NEWUTS,
        }
    }
}

/// open /proc/self/ns/{} of a given namespace and
/// return a file descriptor of specified namespace for the calling process
pub fn open_proc_ns(ns: Namespace) -> Result<RawFd, io::Error> {
    let proc_ns_path = format!("/proc/self/ns/{}", ns.to_proc_ns_name());
    let fd = OpenOptions::new()
        .read(true)
        .open(proc_ns_path)?
        .into_raw_fd();
    Ok(fd)
}

/// open /proc/thread-self/ns/{} of a given namespace and
/// return a file descriptor of specified namespace for the calling thread
pub fn open_thread_ns(ns: Namespace) -> Result<RawFd, io::Error> {
    let task_ns_path = format!("/proc/thread-self/ns/{}", ns.to_proc_ns_name());
    let fd = OpenOptions::new()
        .read(true)
        .open(task_ns_path)?
        .into_raw_fd();
    Ok(fd)
}
