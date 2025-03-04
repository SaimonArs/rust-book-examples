use std::io;

fn main() {
    println!("far: ");
    let mut far = String::new();
    io::stdin()
        .read_line(&mut far)
        .expect("Failed!");

    let far: f64 = match far.trim().parse() {
        Ok(num) => num,
        Err(_) => f64::from(32),
    };
    let cel = (far-f64::from(32))/1.8; 

    println!("{far}F = {cel}C")

}
