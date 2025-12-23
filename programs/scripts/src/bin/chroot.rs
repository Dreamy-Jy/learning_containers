use std::{fs, io::ErrorKind, process::id};

use nix::{
    mount::{MsFlags, mount},
    sched::{CloneFlags, unshare},
    unistd::{Gid, Uid, chdir, chroot, setresgid, setresuid},
};

fn write_file(path: &str, contents: &str) -> Result<(), std::io::Error> {
    fs::write(path, contents.as_bytes())
}

fn main() {
    dbg!(id());

    // Question Block: why does the ordering matter here?
    let uid = Uid::current();
    let gid = Gid::current();

    unshare(CloneFlags::CLONE_NEWUSER | CloneFlags::CLONE_NEWNS | CloneFlags::CLONE_NEWPID)
        .unwrap();
    // Question Block: why does the ordering matter here?
    // Answer: Once your inside the namespace you get new uids and gids
    // so even if your process permissions you wo

    // dbg!(uid);
    // dbg!(gid);
    println!(
        "Outside userns: uid={}, gid={}",
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
        "Inside userns: uid={}, gid={}",
        Uid::current(),
        Gid::current()
    );

    // This works for some reason
    // mount::<str, str, str, str>(None, "/", None, MsFlags::MS_REC | MsFlags::MS_PRIVATE, None)
    //     .unwrap();

    println!("You are root *in the namespace* (mapped to your host uid/gid).");

    chroot("/ubuntu-filesystem").unwrap();
    chdir("/").unwrap();
    // This fails for some reason
    // mount(
    //     Some("proc"), // source: do not pass None here
    //     "/proc",      // target
    //     Some("proc"), // fstype
    //     MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC,
    //     None::<&str>,
    // )
    // .unwrap();
}
