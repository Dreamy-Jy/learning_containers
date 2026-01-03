use std::{fs, io::ErrorKind, process::id};

use nix::{
    mount::{MsFlags, mount},
    sched::{CloneFlags, unshare},
    sys::wait::waitpid,
    unistd::{ForkResult, Gid, Uid, chdir, chroot, fork},
};

fn write_file(path: &str, contents: &str) -> Result<(), std::io::Error> {
    fs::write(path, contents.as_bytes())
}

fn main() {
    dbg!(id());

    // Question Block: why does the ordering matter here?
    let uid = Uid::current();
    let gid = Gid::current();

    println!(
        "Outside userns: uid={}, gid={}",
        Uid::current(),
        Gid::current()
    );

    unshare(CloneFlags::CLONE_NEWUSER | CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_NEWPID)
        .unwrap();
    // Question Block: why does the ordering matter here?
    // Answer: Once your inside the namespace you get new uids and gids
    // so even if your process permissions you wo

    // dbg!(uid);
    // dbg!(gid);
    println!(
        "Inside userns: uid={}, gid={}",
        Uid::current(),
        Gid::current()
    );

    // Question: These can only be set to certain values. Why?
    write_file("/proc/self/uid_map", &format!("0 {} 1\n", uid.as_raw())).unwrap();

    match write_file("/proc/self/setgroups", "deny\n") {
        Ok(_) => {}
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => {
            print!("write /proc/self/setgroups: {:?}", e);
        }
    }

    write_file("/proc/self/gid_map", &format!("0 {} 1\n", gid.as_raw())).unwrap();

    println!(
        "After id mapping userns: uid={}, gid={}",
        Uid::current(),
        Gid::current()
    );

    // This worked for some reason
    // mount::<str, str, str, str>(None, "/", None, MsFlags::MS_REC | MsFlags::MS_PRIVATE, None)
    //     .unwrap();
    match unsafe { fork() } {
        Ok(ForkResult::Child) => {
            // Child is now in the PID namespace
            chroot("/ubuntu-filesystem").unwrap();
            chdir("/").unwrap();

            // This only works in a new process.
            mount(
                Some("proc"), // Question: what is the point of this parameter?
                // Question: ^ : what's the difference between using None::<&str> and Some("proc")
                // here?
                "/proc",
                Some("proc"),
                MsFlags::empty(),
                None::<&str>,
            )
            .unwrap();
        }
        Ok(ForkResult::Parent { child }) => {
            // Parent waits for child
            waitpid(child, None).unwrap();
        }
        Err(e) => panic!("Fork failed: {}", e),
    }
}
