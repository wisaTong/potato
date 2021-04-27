use nix::sys::signal::{self, sigaction, SaFlags, SigAction, SigHandler, SigSet};

/// Block signals for calling thread
/// Change the signal mask of the calling thread through `sigprocmask(2)`.
/// The argument `how` is always set to `SIG_BLOCK`.
/// The set of blocked signals is the union of the current set and the set argument.
pub fn block(set: &[signal::Signal]) {
    let ref sigset = sigset_from_slice(set);
    let how = signal::SigmaskHow::SIG_BLOCK;
    signal::sigprocmask(how, Some(sigset), None).unwrap();
}

/// Unblock signals for calling thread
///
/// Change the signal mask of the calling thread through `sigprocmask(2)`.
/// The argument `how` is always set to `SIG_UNBLOCK`.
/// The signals in set are removed from the current set of blocked signals.
/// It is permissible to attempt to unblock a signal which is not blocked.
pub fn unblock(set: &[signal::Signal]) {
    let ref sigset = sigset_from_slice(set);
    let how = signal::SigmaskHow::SIG_UNBLOCK;
    signal::sigprocmask(how, Some(sigset), None).unwrap();
}

/// Replace the set of blocked signals with the new set for calling thread
///
/// Change the signal mask of the calling thread through `sigprocmask(2)`.
/// The argument `how` is always set to `SIG_SETMASK`.
/// The set of blocked signals is set to the argument set.
pub fn setmask(set: &[signal::Signal]) {
    let ref sigset = sigset_from_slice(set);
    let how = signal::SigmaskHow::SIG_SETMASK;
    signal::sigprocmask(how, Some(sigset), None).unwrap();
}

fn sigset_from_slice(signals: &[signal::Signal]) -> SigSet {
    let mut sigset = SigSet::empty();
    for sig in signals {
        sigset.add(*sig);
    }
    sigset
}

/// Install signal handler for SIGCHLD
pub fn ignore_sigchld() -> Result<(), nix::Error> {
    let handler = SigHandler::SigIgn;
    let sigact = SigAction::new(handler, SaFlags::empty(), SigSet::empty());

    match unsafe { sigaction(signal::SIGCHLD, &sigact) } {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

// This function is fucked up
pub fn set_sa_nocldstop() -> Result<(), nix::Error> {
    let handler = SigHandler::SigDfl;
    let sigact = SigAction::new(handler, SaFlags::SA_NOCLDSTOP, SigSet::empty());

    match unsafe { sigaction(signal::SIGCHLD, &sigact) } {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

extern "C" fn empty(_: libc::c_int) {}

pub fn default_sigcont() -> Result<(), nix::Error> {
    let handler = SigHandler::Handler(empty);
    let sigact = SigAction::new(handler, SaFlags::empty(), SigSet::empty());

    match unsafe { sigaction(signal::SIGCONT, &sigact) } {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn sigsuspend() {
    unsafe { libc::sigsuspend(SigSet::empty().as_ref()) };
}
