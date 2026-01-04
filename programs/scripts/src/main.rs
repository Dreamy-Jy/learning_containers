use std::{
    env,
    ffi::CString,
    fs::{self, create_dir_all, write},
    io::Error,
    os::unix::process::CommandExt,
    path::Path,
    process::{Command, id},
};

use nix::{
    mount::{MsFlags, mount},
    sched::{CloneFlags, clone, unshare},
    sys::{
        signal::Signal,
        wait::{WaitStatus, waitpid},
    },
    unistd::{Gid, Pid, Uid, chdir, chroot, execve},
};

fn main() {
    dbg!(id());
    dbg!(Uid::current());
    dbg!(Gid::current());

    let args: Vec<String> = env::args().collect();
    dbg!(&args);

    match args[1].as_str() {
        "run" => run(),
        "child" => child(),
        _ => panic!("help"),
    }
}

fn run() {
    // check that the program is running as root
    let exe_args: Vec<CString> = {
        let cmd_line_args = env::args()
            .skip(2)
            .map(|a| CString::new(a).unwrap())
            .collect::<Vec<CString>>();
        let mut exe_args = vec![CString::from(c"/proc/self/exe"), CString::from(c"child")];
        exe_args.extend(cmd_line_args);
        exe_args
    };
    let exe_env: Vec<CString> = env::vars_os()
        .map(|(k, v)| {
            let mut kv = k;
            kv.push("=");
            kv.push(v);
            CString::new(kv.into_encoded_bytes()).unwrap()
        })
        .collect();

    dbg!(&exe_args);
    // dbg!(&exe_env);

    let child_pid = unsafe {
        clone(
            Box::new(move || -> isize {
                match execve(c"/proc/self/exe", &exe_args, &exe_env) {
                    Ok(_) => 0,
                    Err(e) => {
                        eprintln!("execve failed: {e}");
                        127
                    }
                }
            }),
            &mut [0u8; 1024 * 1024],
            CloneFlags::CLONE_NEWUSER
                | CloneFlags::CLONE_NEWUTS
                | CloneFlags::CLONE_NEWNS
                | CloneFlags::CLONE_NEWPID,
            Some(Signal::SIGCHLD as i32),
        )
        .unwrap()
    };

    // WARN BLOCK START: Race condition possible here
    // check that the program is root in user namespace
    println!("Child PID: {}", child_pid);

    // When the child process is created it does not have the right permissions in it's user namespace
    // We can't give it the right permissions from within that namespace as the child process
    // As an alternative we give to the child process as the parent process which is running as root.
    write(format!("/proc/{}/uid_map", child_pid), "0 0 1\n").unwrap();
    write(format!("/proc/{}/setgroups", child_pid), "deny\n").unwrap();
    write(format!("/proc/{}/gid_map", child_pid), "0 0 1\n").unwrap();

    cgroup(&child_pid);

    match waitpid(child_pid, None) {
        Ok(WaitStatus::Exited(_, _code)) => {}
        Ok(WaitStatus::Signaled(_, _sig, _)) => {}
        _ => {}
    }
    // WARN BLOCK END: Race condition possible here
}

fn child() {
    // check that the program is root in user namespace
    println!("Entered the child function");
    let args = env::args().collect::<Vec<String>>();

    // check that the new root filesystem exists
    // check that you have the right permissions in the file system
    chroot("/ubuntu").unwrap();
    chdir("/").unwrap();
    mount::<str, str, str, str>(Some("proc"), "proc", Some("proc"), MsFlags::empty(), None)
        .unwrap();

    let _status = Command::new(&args[2])
        .args(&args[3..])
        .status()
        .expect("Failed to execute command");
}

fn cgroup(pid: &Pid) {
    // check which version of cgroups is being used.
    let cgroups = Path::new("/sys/fs/cgroup");
    let liz_dir = cgroups.join("liz");

    create_dir_all(&liz_dir).unwrap();

    let subtree_control = cgroups.join("cgroup.subtree_control");
    if let Ok(controllers) = fs::read_to_string(&subtree_control) {
        if !controllers.contains("pids") {
            // Enable pids controller
            fs::write(&subtree_control, b"+pids").unwrap();
        }
    }

    fs::write(liz_dir.join("cgroup.procs"), pid.to_string().as_bytes()).unwrap();
    // fs::write(liz_dir.join("notify_on_release"), b"1").unwrap(); // only valid in cgroup v1
    fs::write(liz_dir.join("pids.max"), b"20").unwrap();
}

#[allow(dead_code)]
fn old_run() {
    dbg!("Entered the run function");
    dbg!(id());
    let args = env::args().collect::<Vec<String>>();

    let mut cmd = Command::new("/proc/self/exe");

    unsafe {
        cmd.pre_exec(|| {
            dbg!(format!("running unshare before command execution"));
            let result = unshare(
                CloneFlags::CLONE_NEWUSER
                    | CloneFlags::CLONE_NEWUTS
                    | CloneFlags::CLONE_NEWPID
                    | CloneFlags::CLONE_NEWNS,
            )
            .map_err(|e| Error::from_raw_os_error(e as i32));

            dbg!(id());
            result
        });
    }

    let _status = cmd
        .arg("child")
        .args(&args[2..])
        .status()
        .expect("Failed to execute command");
}
