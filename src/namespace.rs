use std::fs::OpenOptions;
use std::io;
use std::os::unix::io::{IntoRawFd, RawFd};

pub enum Namespace {
    Cgroup,
    Ipc,
    Network,
    Mount,
    Pid,
    Time,
    User,
    Uts,
}

impl Namespace {
    fn to_proc_ns_name(&self) -> &str {
        match self {
            Namespace::Cgroup => "cgroup",
            Namespace::Ipc => "ipc",
            Namespace::Network => "net",
            Namespace::Mount => "mnt",
            Namespace::Pid => "pid",
            Namespace::Time => "time",
            Namespace::User => "user",
            Namespace::Uts => "uts",
        }
    }
}

/// get file descriptor of specified namespace for the calling process
pub fn get_proc_ns_fd(ns: Namespace) -> Result<RawFd, io::Error> {
    let proc_ns_path = format!("/proc/self/ns/{}", ns.to_proc_ns_name());
    let fd = OpenOptions::new()
        .read(true)
        .open(proc_ns_path)?
        .into_raw_fd();
    Ok(fd)
}

/// get file descriptor of specified namespace for the calling thread
pub fn get_thread_ns_fd(ns: Namespace) -> Result<RawFd, io::Error> {
    let task_ns_path = format!("/proc/thread-self/ns/{}", ns.to_proc_ns_name());
    let fd = OpenOptions::new()
        .read(true)
        .open(task_ns_path)?
        .into_raw_fd();
    Ok(fd)
}
