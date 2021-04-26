use libc::{pid_t, uid_t};
use nix::errno::{errno, Errno};
use std::fs;

pub struct UidMapper {
    mappings: Vec<UidMap>,
}

struct UidMap {
    uid: uid_t,
    lower_uid: uid_t,
    count: uid_t,
}

impl UidMapper {
    /// construct new `UidMapper` with empty mappings
    pub fn new() -> UidMapper {
        UidMapper {
            mappings: Vec::new(),
        }
    }

    /// map user id between user namespaces
    ///
    /// * `uid` - Beginning of the range of UIDs inside the user namespace.
    /// * `lower_uid` - Beginning of the range of UIDs outside the user namespace.
    /// * `count` - Length of the ranges (both inside and outside the user namespace).
    pub fn add(mut self, uid: uid_t, lower_uid: uid_t, count: uid_t) -> Self {
        let uid_map = UidMap {
            uid,
            lower_uid,
            count,
        };
        self.mappings.push(uid_map);
        self
    }

    /// write to `/proc/[pid]/uid_map`. See newuidmap(1) and user_namespaces(7)
    ///
    /// `/proc/[pid]/uid_map` can only be written to once for all process
    /// in the same user namespace as `pid`.
    pub fn write_newuidmap(self, pid: pid_t) -> Result<(), Errno> {
        let mut content = String::new();
        for map in self.mappings {
            let line = format!("{} {} {}\n", map.uid, map.lower_uid, map.count);
            content.push_str(&line);
        }

        let path = format!("/proc/{}/uid_map", pid);
        fs::OpenOptions::new().write(true).open(&path).unwrap();
        match fs::write(path, content.as_bytes()) {
            Ok(_) => Ok(()),
            Err(_) => Err(Errno::from_i32(errno())),
        }
    }
}
