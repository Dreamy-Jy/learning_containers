# Session 1 Plan

## Goals

Get a broad overview of how containers are materialized by the OS.
Write a program that:

- CRUDs containers
- Has a comprehensive test plan

Support

- [Writing a container in Rust](https://litchipi.site/serie/containers_in_rust)
- [Linux containers in 500 lines of code](https://blog.lizzie.io/linux-containers-in-500-loc.html)
- [Build Your Own Container Using Less than 100 Lines of Go](https://www.infoq.com/articles/build-a-container-golang/)
- [Learning Containers From The Bottom Up](https://iximiuz.com/en/posts/container-learning-path/)

## Limits

I will not be reverse engineering systems like Docker & Kubernetes
I will not be creating production ready technology

## Program Plan

### Container Tutorial Redo

- Check the version of Linux
- Generate a hostname
- Create isolated execution environment in child process
  - External Setup: have new namespaces
  - Internal Setup:
    - set hostname
    - unmount things this process doesn't need
    - set program to run in new user namespace
    - limit resources with CGroups
    - limit capabilities
    - disallow syscalls with in the environment
  - Execute program at path configured

### Container CRUD Service

Service is made of a Daemon & CLI Interface. The Deamon performs Container CRUD operations, while the CLI Interface tells the Daemon what to do.

- CLI Interface
  - Deamon CRUD Operations
    - Start
    - Stop
    - Status
    - Update
  - Container Operations
    - Create
    - Read
    - Update
    - Destroy
- Deamon
  - Container Operation Algorithms:
    - Create
    - Read
    - Update
    - Destroy

## Furthure Study

Using wasm as container instead of linux.
How to Hack ill configured linux system.

- What can processes do.
- Can I access memory of another program.
- How else can I hack these.
