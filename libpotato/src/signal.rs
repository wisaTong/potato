use core::convert::TryFrom;
use nix::sys::signal::{self, SigSet};
use nix::sys::wait;

/// Block signals for calling thread
///
/// Change the signal mask of the calling thread through `sigprocmask(2)`.
/// The argument `how` is always set to `SIG_BLOCK`.
/// The set of blocked signals is the union of the current set and the set argument.
pub fn block(set: &[signal::Signal]) -> Result<(), nix::Error> {
    let ref sigset = sigset_from_slice(set);
    let how = signal::SigmaskHow::SIG_BLOCK;
    signal::sigprocmask(how, Some(sigset), None)
}

/// Unblock signals for calling thread
///
/// Change the signal mask of the calling thread through `sigprocmask(2)`.
/// The argument `how` is always set to `SIG_UNBLOCK`.
/// The signals in set are removed from the current set of blocked signals.
/// It is permissible to attempt to unblock a signal which is not blocked.
pub fn unblock(set: &[signal::Signal]) -> Result<(), nix::Error> {
    let ref sigset = sigset_from_slice(set);
    let how = signal::SigmaskHow::SIG_UNBLOCK;
    signal::sigprocmask(how, Some(sigset), None)
}

/// Replace the set of blocked signals with the new set for calling thread
///
/// Change the signal mask of the calling thread through `sigprocmask(2)`.
/// The argument `how` is always set to `SIG_SETMASK`.
/// The set of blocked signals is set to the argument set.
pub fn setmask(set: &[signal::Signal]) -> Result<(), nix::Error> {
    let ref sigset = sigset_from_slice(set);
    let how = signal::SigmaskHow::SIG_SETMASK;
    signal::sigprocmask(how, Some(sigset), None)
}

fn sigset_from_slice(signals: &[signal::Signal]) -> SigSet {
    let mut sigset = SigSet::empty();
    for sig in signals {
        sigset.add(*sig);
    }
    sigset
}

extern "C" fn handle_sigchld(_: libc::c_int) {
    wait::wait().unwrap();
}

extern "C" fn handle_sigchld_exit(_: libc::c_int) {
    wait::wait().unwrap();
    std::process::exit(0);
}

pub fn install_naive_signal_handler(
    signal: i32,
    handler: extern "C" fn(libc::c_int),
    saflags: signal::SaFlags,
) -> Result<(), nix::Error> {
    let handler = signal::SigHandler::Handler(handler);
    let sigaction = signal::SigAction::new(handler, saflags, signal::SigSet::empty());
    let sig = signal::Signal::try_from(signal)?;

    match unsafe { signal::sigaction(sig, &sigaction) } {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

/// Install signal handler for SIGCHLD
pub fn install_sigchld_handler() -> Result<(), nix::Error> {
    install_naive_signal_handler(libc::SIGCHLD, handle_sigchld, signal::SaFlags::SA_NOCLDSTOP)
}

pub fn install_sigchld_handler_exit() -> Result<(), nix::Error> {
    install_naive_signal_handler(
        libc::SIGCHLD,
        handle_sigchld_exit,
        signal::SaFlags::SA_NOCLDSTOP,
    )
}
