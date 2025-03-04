use std::collections::HashMap;



fn pig_latin(stre: &str) -> String{
    let alph: HashMap<char, char> = HashMap::from([('a', '?'),('e', '?'),('i', '?'), ('o', '?'),('u', '?')]);
    let stree = stre.trim().to_ascii_lowercase();
    let v: Vec<&str> = stree.split(' ').collect();
    let mut vv: Vec<String> = Vec::new();
    for j in v.iter() {
        let c = j.chars().next().unwrap();
        match alph.get(&c) {
            None => {vv.push(format!("{}-{}ay", &j[1..], c))},
            Some(_) => {vv.push(format!("{}-hay", &j[..]))},
        }
    }
    return vv.join(" ")
    
}

fn main() {
    println!("{}", pig_latin("In Rust I couldnt find a similar method as replacement Something I came up with looks like this"));
}
