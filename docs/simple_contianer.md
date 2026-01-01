
I should have created an algorithm for containerization and built from there.

Algorithm, Tests, Implementation

## Issues

### chroot(2) fails with EPERM

#### Issue Reframe

- chroot(2) fails with EPERM
- can't create a independent filesystem

There's a chance that I'ven't setup the process correctly for the OS to allow
me to change the root directory.

There's a chance that the process I'ven't set up the process with the right permissions.

Instead of chroot I could do a pivot_root.

Not being able to change root because child process is not privileged
The core problem here is that I don't Linux well enough to by pass security. 
The child process I'm spawning can't change it's root directory.

I'm running this program on an ubuntu VM and alpine container.

#### Attempted Solutions

Instead of chroot I could use a pivot_root

I can use chroot when running the executable as root/sudo

## Further Study

Why can we change the file system?
Write a program that can execute programs securely.
What is the difference between these 2:
```rust
    // map id
    write_file("/proc/self/uid_map", &format!("0 {} 1\n", uid))?;
    write_file("/proc/self/gid_map", &format!("0 {} 1\n", gid))?;

    // set id
    setresgid(Gid::from_raw(0), Gid::from_raw(0), Gid::from_raw(0))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("setresgid: {e}")))?;
    setresuid(Uid::from_raw(0), Uid::from_raw(0), Uid::from_raw(0))
        .map_err(|e| io::Error::new(io::ErrorKind::Other, format!("setresuid: {e}")))?;

```

---
Namespaces
Control Groups

chroot
chdir
mount
unshare
fork
clone
/proc/self/(uid_map | setgroups | gid_map)

Concepts:
- explain the concepts
	- from a manual pages perspective
	- scripts for experiments
- from a kernel code perspective

# Background
## Linux Namespaces
Relevant Man Pages
[namespace(7)](https://man7.org/linux/man-pages/man7/namespaces.7.html)
[mount_namespaces(7)](https://man7.org/linux/man-pages/man7/mount_namespaces.7.html)
[user_namespaces(7)](https://man7.org/linux/man-pages/man7/user_namespaces.7.html)
[pid_namespaces(7)](https://man7.org/linux/man-pages/man7/pid_namespaces.7.html)

To my understanding namespaces isolate process usage of global resources from processes in other namespaces.

Namespaces are interacted with via system calls or the proc filesystem.

Namespaces can automatically shutdown, but for a number of reasons can outlive your program. Make sure to take down all resources blocking namespace shutdown before closing your program. Check [namespace(7): Namespace lifetime](https://man.archlinux.org/man/namespaces.7.en#Namespace_lifetime) for more details.
### General Namespace
#### Namespace Overview
These are all of the namespaces listed in Linux man-pages 6.16

| **Namespace** | **Flag**            | **Page**                                                                       | **Isolates**                         |
| ------------- | ------------------- | ------------------------------------------------------------------------------ | ------------------------------------ |
| Cgroup        | **CLONE_NEWCGROUP** | [cgroup_namespaces(7)](https://man.archlinux.org/man/cgroup_namespaces.7.en)   | Cgroup root directory                |
| IPC           | **CLONE_NEWIPC**    | [ipc_namespaces(7)](https://man.archlinux.org/man/ipc_namespaces.7.en)         | System V IPC, POSIX message queues   |
| Network       | **CLONE_NEWNET**    | [network_namespaces(7)](https://man.archlinux.org/man/network_namespaces.7.en) | Network devices, stacks, ports, etc. |
| Mount         | **CLONE_NEWNS**     | [mount_namespaces(7)](https://man.archlinux.org/man/mount_namespaces.7.en)     | Mount points                         |
| PID           | **CLONE_NEWPID**    | [pid_namespaces(7)](https://man.archlinux.org/man/pid_namespaces.7.en)         | Process IDs                          |
| Time          | **CLONE_NEWTIME**   | [time_namespaces(7)](https://man.archlinux.org/man/time_namespaces.7.en)       | Boot and monotonic clocks            |
| User          | **CLONE_NEWUSER**   | [user_namespaces(7)](https://man.archlinux.org/man/user_namespaces.7.en)       | User and group IDs                   |
| UTS           | **CLONE_NEWUTS**    | [uts_namespaces(7)](https://man.archlinux.org/man/uts_namespaces.7.en)         | Hostname and NIS domain name         |
#### Namespace Related Syscall Overview
[clone(2)](https://man.archlinux.org/man/clone.2.en) - Creates a child process inside of a set of new namespaces as defined by an input parameter flag
[setns(2)](https://man.archlinux.org/man/setns.2.en) - moves the calling process into a existing namespace(s).
[unshare(2)](https://man.archlinux.org/man/unshare.2.en) - moves the calling process into new namespace(s).
[ioctl(2)](https://man.archlinux.org/man/ioctl.2.en) - can used to get information about a namespace.
#### Namespace Related Filesystem Overview
`/proc/<pid>/ns/`
These files are symlinks to for namespaces that can be manipulated with setns(2). Each file represents a namespace a process or it's children could be apart of.
- _cgroup_
- _ipc_
- _mnt_
- _net_
- _pid_
- _pid_for_children_
- _time_
- _time_for_children_
- _user_
- _uts_
`/proc/sys/user/`
These are files set limits on the amount of namespaces that can be created.
- _max_cgroup_namespaces_
- _max_ipc_namespaces_
- _max_mnt_namespaces_
- _max_net_namespaces_
- _max_pid_namespaces_
- _max_time_namespaces_
- _max_user_namespaces_
- _max_uts_namespaces_
### User Namespaces
"User namespaces isolate security-related identifiers and attributes, in particular, user IDs and group IDs (see [credentials(7)](https://man.archlinux.org/man/credentials.7.en)), the root directory, keys (see [keyrings(7)](https://man.archlinux.org/man/keyrings.7.en)), and capabilities (see [capabilities(7)](https://man.archlinux.org/man/capabilities.7.en))." - Man Page
#### Notes
- User namespaces can be nested. There is a limit to this.
- A process can only be in one user name space.
- Your process doesn’t need to be privileged to create a new user namespace, but does require privileges to create any other kind of namespace.
- When trying to access a file, the system will map the files, credentials, and your process of credentials back to what they would be in the root user name space.
    - This behavior can be overrated with capabilities.
- **Project IDs** effect setting disk quotas. [setquota(8)](https://man.archlinux.org/man/setquota.8.en) and [quotactl(2)](https://man.archlinux.org/man/quotactl.2.en) can be used for this.
##### U/GID Mappings
- Processes' U/GIDs are initially unmapped to within that user namespace they are added to.
	- You must manually map them via writing to UID and GID map files(`/proc/<pid>/(uid_map | gid_map)`), you can only do this operation once.
		- The result of reading uid_map or gid_map files are relative to the process accessing them.
	- In order to use system calls that modify UID or GID, Both values must be mapped locally.
        - For mapping GID you may need to set the set groups capability to deny, if your process does not have the CAP_SETGID capability.
    - Be aware that unmapped IDs can show up within a variety of system interfaces.
- Processes can have different U/GIDs in and out of a namespace.
	- Unprivileged processes can be privileged in a namespace.
- **Unmapped U/GID**: When using operations that return U/GIDs that are not mapped within your user namespace you may get the overflow(65534) U/GID.
- **Passing U/GID Over Sockets**: When using sockets (check **SCM_CREDENTIALS** in[unix(7)](https://man.archlinux.org/man/unix.7.en)) to communicate uid and gids between process in different namespaces there is translation to the frame of reference of the receiving process.
##### Capabilities
- When a process is added to a namespace it gains a full set of capabilities within that name space. This set of capabilities can be recalculated though.
    - User Namespace capabilities are hierarchical and cascading. Processes in a parent user namespace have the same capabilities of that user namespace in child user namespaces.
    - Your process will only have access to privileged actions for resources owned by the user namespace. Your process needs to be privileged in the root user namespace to perform these privileged actions.
%% 
#### Related System Calls
[unshare(2)](https://man.archlinux.org/man/unshare.2.en) or [clone(2)](https://man.archlinux.org/man/clone.2.en) - 
[fork(2)](https://man.archlinux.org/man/fork.2.en)
[setns(2)](https://man.archlinux.org/man/setns.2.en)
[ioctl(2)](https://man.archlinux.org/man/ioctl.2.en)
[ioctl_nsfs(2)](https://man.archlinux.org/man/ioctl_nsfs.2.en)
[prctl(2)](https://man.archlinux.org/man/prctl.2.en)
[setgroups(2)](https://man.archlinux.org/man/setgroups.2.en)
[getuid(2)](https://man.archlinux.org/man/getuid.2.en), [getgid(2)](https://man.archlinux.org/man/getgid.2.en) [stat(2)](https://man.archlinux.org/man/stat.2.en), [waitid(2)](https://man.archlinux.org/man/waitid.2.en)
#### Related File Systems 
%%

### Further Study
```js
// script used to collect man page links from man page.
const unique_links = [...new Set([...document.querySelectorAll("p > a")].map(e => e.text))];
const regex = /(?:\w+_)?[A-Za-z]+\([0-9]\)/;
unique_links.filter(e => regex.test(e))
```
### Mount Namespaces
[mount_namespaces(7)](https://man.archlinux.org/man/mount_namespaces.7.en)
### PID Namespaces
[pid_namespaces(7)](https://man.archlinux.org/man/pid_namespaces.7.en)

PID Namespaces isolate local PID with a that specific PID namespace. This is useful for preforming operations on a collective subset of processes running on your system.

Use of PID namespaces requires a kernel that is configured with the **CONFIG_PID_NS** option.
#### Notes
- The first process in a new PID namespace functions like the init/systemd ([init(1)](https://man.archlinux.org/man/init.1.en)) root process.
	- It adopts orphaned processes in the namespace. You can change this behavior using [prctl(2)](https://man.archlinux.org/man/prctl.2.en) **PR_SET_CHILD_SUBREAPER**.
	- If the PID Namespace's init process exits, all processes are killed via `SIGKILL`.
	- Child processes can't send signals to the init process, if the init process doesn't have a signal handler for that signal.
	- When an init process or process in a parent pid namespace send kill and stop signals PID Namespaces cascade them forcibly. 
- PID namespaces are nested.
- Processes have unique PIDs in each namespace they are visible to. Processes in child PID namespaces can't view processes in their ancestor PID namespaces.
	- Not sure how deep ancestors can peer into their descendants. 
- When moving processes between PID namespaces, processes can only descent into it's descendants. 
- Use the **NS_GET_PARENT** [ioctl(2)](https://man.archlinux.org/man/ioctl.2.en) to discover the relationships between PID namespaces.
- A process can only call [unshare(2)](https://man.archlinux.org/man/unshare.2.en) to create a new pid namespace once.
	- something about emptying the `/proc/<pid>/ns/pid_for_children` symlink
- **CLONE_NEWPID** is not allowed with the follow **CLONE_(*)** flags:
	- **CLONE_THREAD**
	- **CLONE_SIGHAND**
	- **CLONE_VM**
- the `/proc` filesystem shows all processes visible to the PID namespace of the process that mounts it.
	- child process could view parent process
	- Its recommend to change root and mount a new proc to change this.
	- If you create a new mount namespace you may be able to not need to change namespace
- `/proc/sys/kernel/ns_last_pid` - shows the last PID that's been created in the PID namespace of the process calling it.

#### Experiments
- One way to hit this is `setns()` into a PID namespace whose PID 1 already terminated (e.g., using an open `/proc/<pid>/ns/pid` FD), after which process creation fails with `ENOMEM`.
- Another way to hit this is calling `unshare(CLONE_NEWPID)` and then having the first `fork()`’d child (the would-be PID 1) exit, causing later `fork()` calls to fail with `ENOMEM`.
- understand change root,
- understand mount
- check facts on `/proc/sys/kernel/ns_last_pid`
- test how far into their descendants a ancestor PID Namespaces can view. 
- use ioctl(2) and prctl(2) operations.
- test that [unshare(2)(CLONE_NEWPID)](https://man.archlinux.org/man/unshare.2.en) can only be called once per process.
- make unshare and clone fail with a combo of bad flags.
### UTS Namespace
Isolates the hostname and the NIS domain name. 
- When you create a new UTS namespace, you inherit the hostname and NIS domain name from the parent UTS namespace.
- To use UTS namespaces you must configure the kernel with **CONFIG_UTS_NS**.
#### Related System Calls
[sethostname(2)](https://man.archlinux.org/man/sethostname.2.en) 
[setdomainname(2)](https://man.archlinux.org/man/setdomainname.2.en)
[uname(2)](https://man.archlinux.org/man/uname.2.en)
[gethostname(2)](https://man.archlinux.org/man/gethostname.2.en)
[getdomainname(2)](https://man.archlinux.org/man/getdomainname.2.en)

---

Mounts
How would I know which file systems to mount?

Get Non-Driver ID, Get Library Card.