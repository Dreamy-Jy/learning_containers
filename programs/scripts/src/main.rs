use std::env;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args[1].as_str() {
        "run" => run(),
        "child" => child(),
        _ => panic!("help"),
    }

    dbg!(args);
}

fn run() {
    println!("From run() function:\n\trunning the main coroutine");
}

fn child() {
    println!("From child() function:\n\trunning the child coroutine");
}
