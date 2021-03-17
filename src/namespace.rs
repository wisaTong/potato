use crate::error::Error;
use nix::sched::{setns, unshare, CloneFlags};
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io;
use std::os::unix::io::{IntoRawFd, RawFd};
use std::vec::Vec;

#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
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
    pub fn to_proc_ns_name(&self) -> &str {
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

pub fn create_nses(nses: Vec<Namespace>) -> Result<HashMap<Namespace, RawFd>, Error> {
    let mut clone_flag = CloneFlags::empty();
    let mut original_ns_fds = HashMap::new();
    for ns in nses.clone() {
        clone_flag = clone_flag | ns.to_clone_flag();
        original_ns_fds.insert(ns, get_proc_ns_fd(ns)?);
    }

    // creating new namespaces
    unshare(clone_flag)?;
    let mut ns_fds = HashMap::new();
    for ns in nses.clone() {
        ns_fds.insert(ns, get_proc_ns_fd(ns)?);
    }

    // restoring orignal namespaces
    for (ns, fd) in &original_ns_fds {
        setns(*fd, ns.to_clone_flag())?;
    }

    Ok(ns_fds)
}
