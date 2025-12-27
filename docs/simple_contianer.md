
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
User namespaces isolate security-related identifiers and attributes, in particular, user IDs and group IDs (see [credentials(7)](https://man.archlinux.org/man/credentials.7.en)), the root directory, keys (see [keyrings(7)](https://man.archlinux.org/man/keyrings.7.en)), and capabilities (see [capabilities(7)](https://man.archlinux.org/man/capabilities.7.en)).

- A process can have different user and group IDs inside and outside of a user namespace
    - This means an unprivileged process can be privileged within a namespace.
- User Name spaces can be nested. There is a limit to this.
- A process can only be in one user name space.
- When a process is added to a name, space it gains a full set of capabilities within that name space. This set of capabilities can be recalculated though.
    - User Namespace capabilities are hierarchical. Processes in a parent user namespace have the same capabilities of that user namespace in child user namespaces.
    - Your process will only have access to privileged actions for resources owned by the user namespace. your process needs to be privileged in the root user namespace to perform these privileged actions.
- Your process doesn’t need to be privileged to create a new user namespace, but does require privileges to create any other kind of namespace. There are some provided workarounds within syscalls to allow unprivileged processes to create a full suite of namespaces.
- When creating an entering a user name, space, your processes user ID and group ID are not automatically mapped to local user in group IDs.
    - You must manually map them via writing to UID and GID map files, you can only do this operation once.
        - Different processes may not get the same value reading map files. Reading from a processes, UID or GID map files provides data relative to the frame of reference of the process reading that file.
    - In order to use system calls that modify UID or GID, Both values must be mapped locally.
        - For mapping GID you may need to set the set groups capability to deny, if your process does not have the CAP_SETGID capability.
    - Be aware that unmapped IDs can show up within a variety of system interfaces.
- Project IDs Can also be mapped within the name space. I have no clue what these are what they do nor any interest in finding out.
- When trying to access a file, the system will map the files, credentials, and your process of credentials back to what they would be in the root user name space.
    - This behavior can be overrated with capabilities.

Where is the process data structure within the kernel?

Man Page Perspective
- Overview and Notes
- Relevant System Calls and Files
Kernel Perspective
- What are the Kernel data structures.
- What subsystems are these structure used in.
### Mount Namespaces

### PID Namespaces

---

Mounts
How would I know which file systems to mount?

