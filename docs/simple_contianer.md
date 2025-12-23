
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

I can use chroot when running the excutible as root/sudo

## Furthur Study

Why can we change the file system?
Write a program that can execute programs securely.
