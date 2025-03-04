use std::io;

fn main() {
    let mut n = String::new();

    println!("input n: ");

    io::stdin()
        .read_line(&mut n)
        .expect("Failed");

    let n: u128 = match n.trim().parse() {
        Ok(num) => num,
        Err(_) => 1,
    };

    let x = fib(n);
    println!("n = {n}, fib = {x}");
}

fn fib(n: u128) -> u128 {
    match n {
        0 => 0,
        1 => 0,
        2 => 1,
        _ => fib(n-1)+fib(n-2),
    }
}
