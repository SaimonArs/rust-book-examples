use std::{collections::HashMap, io};

fn main() {
    let mut companys: HashMap<String, Vec<String>> = HashMap::new();
    
    loop {
        let mut qu = String::new();

        io::stdin()
            .read_line(&mut qu)
            .expect("Failed to read line");
        
        let mut iter = qu.split_ascii_whitespace();

        match iter.next().expect("None").to_ascii_lowercase().as_str() {
            "add" => {
                let mut op = Vec::new();
                for _ in 0..3 {
                    let it = iter.next().expect("None");
                    match it {
                        "None" => {println!("error!"); break;},
                        _ => op.push(it),
                    }
                }
                if op.len() != 3 {
                    println!("error!");
                    continue;
                }
                let d = op[2].to_string();
                if companys.get(&d) == None {
                    let mut vect = Vec::new();
                    vect.push(op[0].to_string());
                    companys.insert(d, vect);
                } else {
                    let mut dc =companys.get(&d).unwrap().to_vec(); 
                    dc.push(op[0].to_string());
                    companys.insert(d, dc);
                }
            },
            "show" => {
                let it = iter.next();
                match it {
                    None => {
                        for j in companys.iter() {
                            println!("##{}: ", j.0);
                            let mut vect = j.1.clone();
                            vect.sort_by_key(|k| k.chars().next().unwrap() as u32);
                            println!("{}", vect.join("\n"));
                        }

                    },
                    _ => {
                        let odc =companys.get(&(it.unwrap()).to_string());
                        match odc {
                            None => {
                                println!("error");
                                continue;
                            }
                            _ => {
                               let mut dc = odc.unwrap().clone();
                               dc.sort_by_key(|k| k.chars().next().unwrap() as u32);
                               println!("{}", dc.join("\n")) 
                            }
                        }
                    }
                }

            },
            _ => {},
            
        }

    }
}
