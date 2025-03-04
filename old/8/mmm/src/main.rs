use std::collections::HashMap;


fn mmm(vecte: &Vec<i32>) -> (f32, f32, f32) {
    let mut acc = 0;
    let mut amax = i32::MIN;
    let mut imax: i32 = 0;
    let n = vecte.len();
    let mut ccou: HashMap<i32, i32> = HashMap::new();
    let mut vect = vecte.clone();

    vect.sort();
    

    for (_, x) in vect.iter().enumerate() {
        acc += *x;
        let mut num = ccou.get(x);
        ccou.insert(*x, match num {
            None => 1,
            _ => *num.unwrap() + 1,    
        });
        num = ccou.get(x);
        if amax < *num.unwrap() {
            imax = *x;
            amax = *num.unwrap();
        }
    }

    return ((acc as f32)/(n as f32), vect[n / 2] as f32, imax as f32)

}

fn main() {
    let x = vec![1, 2, 3, 4, 2, 4, 4];
    let a = mmm(&x);
    println!("{a:?}");
}
