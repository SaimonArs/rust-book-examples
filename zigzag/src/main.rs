fn main() {
    let a = convert(String::from("PAYPALISHIRING"), 4);
    println!("{}", a);
    //let s: &str = "hello";
    //println!("{}", s.chars().nth(0).unwrap());
}

fn convert(s: String, num_rows: i32) -> String{
    let mut ans = String::new();
    let step = (num_rows-1)*2;
    if num_rows <= 1 {
        println!("{}", s);
    } else {
        for i in 0..num_rows {
            let mut j = i;
            let mut nex = step - (i*2)%step;
            while j < s.len() as i32{
                ans.push(s.chars().nth(j as usize).expect("errror"));
                j = j + nex;
                nex = step - nex % step;
            }
        }
    }
    ans
}
