use nix::sched::{setns, unshare, CloneFlags};
use nix::unistd;
use potato::namespace::{self, Namespace};
use std::thread;

fn main() {
    let original_fd = namespace::get_proc_ns_fd(Namespace::Uts).expect("Problem getting ns fd");
    print_hostname();

    let mut ns_fds = vec![];
    for _ in 0..3 {
        // unsharing new namespaces
        unshare(CloneFlags::CLONE_NEWUTS).expect("Problem unsharing");
        println!("Unshare UTS namespaces");

        let fd = namespace::get_proc_ns_fd(Namespace::Uts).expect("Problem getting ns fd");
        ns_fds.push(fd);

        // restoring original namespaces
        setns(original_fd, CloneFlags::empty()).expect("Problem restoring ns");
    }

    let mut threads = vec![];
    for fd in ns_fds {
        let t = thread::spawn(move || {
            setns(fd, CloneFlags::empty()).expect("Problem attempting to setns");
            unistd::sethostname(fd.to_string()).expect("Problem setting hostname");
            print_hostname();
        });
        threads.push(t);
    }

    // join thread (waiting for them to finish executing)
    for t in threads {
        t.join().unwrap();
    }

    // print hostname from main thread to show that it has not been effected
    print_hostname();
}

fn print_hostname() {
    let mut buf = [0u8; 64];
    let hostname_cstr = unistd::gethostname(&mut buf).expect("Failed getting hostname");
    let hostname = hostname_cstr.to_str().expect("Hostname wasn't valid UTF-8");
    println!("Hostname: {}", hostname)
}
