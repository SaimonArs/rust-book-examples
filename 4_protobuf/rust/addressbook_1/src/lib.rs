pub mod arguments;
mod pb {
    include!(concat!(env!("OUT_DIR"), "/addressbook.ab.rs"));
}

use std::fs;
use std::io::{BufReader, Read, Write, BufWriter};
use std::error::Error;
use pb::company::Department;
use pb::contact::Kind;
use pb::person::phone_number::Type;
use prost::Message;
use prost_types::Timestamp;
use std::time;

const DB_FILE_PATH: &str = "addressbook.db";

use arguments::{Cli, Commands, DepType, KindType, PhoneType};

pub fn run(config: Cli) -> Result<(), Box<dyn Error>> {
    let mut f = open_db_file(DB_FILE_PATH);
    match &config.command {
        Some(Commands::Add(x)) => {
            let name = x.name.as_str();
            let email = match &x.email {
                Some(em) => em.as_str(),
                None => ""
            };
            let phone = match &x.phone {
                Some(em) => em.as_str(),
                None => ""
            };
            match x.kind {
                KindType::Cie | KindType::Company => {
                    add_company(&mut f, name, email, x.dep.clone(), phone, x.dep.clone());
                    }
                KindType::Per | KindType::Person => {
                    add_person(&mut f, name, email, phone, x.r#type.clone());
                    }
            }
            }
        Some(Commands::List(x)) => {
            list_contacts(&mut f, x.redact);
            }
        None => Err("Command not found")?
    }
    Ok(())
}

fn open_db_file(file_path: &str) -> fs::File {
    // File::open only for reading
    fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true) // Creates if file does not exists
        .open(file_path)
        .unwrap()
}

fn read_from_db(f: &mut fs::File) -> pb::AddressBook {
    let mut buf_reader = BufReader::new(f);
    let mut contents = Vec::new();
    buf_reader.read_to_end(&mut contents).unwrap();
    pb::AddressBook::decode(contents.as_slice()).unwrap()
}

fn write_to_db(f: &mut fs::File, book: pb::AddressBook) {
    let mut buf_writer = BufWriter::new(f);
    let contents = book.encode_to_vec();
    buf_writer.write_all(&contents).unwrap();
    buf_writer.flush().unwrap();
}
fn str_to_phone_type(t: PhoneType) ->i32 {
    match t {
        PhoneType::Home => 2,
        PhoneType::Mobile => 1,
        PhoneType::Work => 3,
        _ => 0,
    }
}
fn str_to_department(t: DepType) -> i32 {
    match t {
        DepType::Hr => 1,
        DepType::Cs => 2,
        _ => 0
    }
}

fn add_person(f: &mut fs::File, name: &str, email: &str, phone: &str, phone_type: PhoneType) {
    let mut book = read_from_db(f);
    let mut person: pb::Person;
    if book.contacts.contains_key(name) {
        // If Kind exists
        let kind = book.contacts.get(name).unwrap().kind.clone().unwrap();
        match kind {
            pb::contact::Kind::Person(p) => { person = p },
            pb::contact::Kind::Company(_) => {panic!("Company")},
        }
    }
    else {
        person = pb::Person::default();
    }
    person.email = email.to_string();
    let mut nb = pb::person::PhoneNumber::default();
    nb.number = phone.to_string();
    nb.r#type = str_to_phone_type(phone_type);
    person.phones.push(nb);

    let mut contact = pb::Contact::default();
    let mut update_ts = Timestamp::default();
    let duration = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap();
    update_ts.seconds = duration.as_secs() as i64;
    update_ts.nanos = duration.subsec_nanos() as i32;

    contact.last_updated = Some(update_ts);
    contact.kind = Some(pb::contact::Kind::Person(person));
    book.contacts.insert(String::from(name), contact);

    write_to_db(f, book);
}

fn add_company(f: &mut fs::File, name: &str, email: &str, email_dep: DepType, phone: &str, phone_dep: DepType) {
    let mut book = read_from_db(f);

    let mut company: pb::Company;
    if book.contacts.contains_key(name) {
        // If Kind exists
        let kind = book.contacts.get(name).unwrap().kind.clone().unwrap(); // .clone
        match kind {
            pb::contact::Kind::Company(comp) => {company = comp },
            pb::contact::Kind::Person(_) => { panic!("Person") },
        }
    }
    else {
        company = pb::Company::default();
    }

    let mut addr = pb::company::EmailAddress::default();
    addr.email = email.to_string();
    addr.department = str_to_department(email_dep);
    company.emails.push(addr);

    let mut nb = pb::company::PhoneNumber::default();
    nb.number = phone.to_string();
    nb.department = str_to_department(phone_dep);
    company.phones.push(nb);

    let mut contact = pb::Contact::default();
    let mut update_ts = Timestamp::default();
    let duration = time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap();
    update_ts.seconds = duration.as_secs() as i64;
    update_ts.nanos = duration.subsec_nanos() as i32;

    contact.last_updated = Some(update_ts);
    contact.kind = Some(pb::contact::Kind::Company(company));
    book.contacts.insert(String::from(name), contact);

    write_to_db(f, book);
}

fn list_contacts(f: &mut fs::File, redact: bool) {
    let book = read_from_db(f);
    let mut keys: Vec<&String>  = book.contacts.keys().collect();
    keys.sort();

    for name in keys {
        let mut contact = book.contacts.get(name).unwrap().clone();
        if redact {
            match &mut contact.kind {
                &mut Some(Kind::Person(ref mut p)) => {
                    p.phones.iter_mut().for_each(|pn| 
                        pn.number = pn.number.chars().map(|_| '*').collect()
                    );
                    p.email = p.email.chars().map(|_| '*').collect()
                }
                &mut Some(Kind::Company(ref mut c)) => {
                    c.phones.iter_mut().for_each(|cn|
                        cn.number = cn.number.chars().map(|_| '*').collect()
                    );

                    c.emails.iter_mut().for_each(|ce|
                        ce.email = ce.email.chars().map(|_| '*').collect()
                    );
                }
                &mut None => {}
                
            }
        }
        
        println!("last_updated: {:?}", chrono::DateTime::from_timestamp_nanos(contact.last_updated.unwrap().seconds * 1000000000));
        match contact.kind {
            Some(Kind::Person(p)) => {
                    println!("kind: Person");
                    println!("name: {}", name);
                    println!("email: {}", p.email);
                    println!("phones: \n ---- ");
                    p.phones.iter().for_each(|pn| {
                        if let Ok(phone_type) = Type::try_from(pn.r#type) {
                            println!("phone: {} \n type: {:?}", pn.number, phone_type);
                        }
                    });
                }
                Some(Kind::Company(c)) => {
                    println!("kind: Company");
                    println!("name: {}", name);
                    println!("emails: \n ----");
                    c.emails.iter().for_each(|em| {
                        if let Ok(department) = Department::try_from(em.department) {
                            println!("email: {} \n department: {:?}", em.email, department);
                        }
                    });
                    println!("phones: \n ---- ");
                    c.phones.iter().for_each(|pn| {
                        if let Ok(department) = Department::try_from(pn.department) {
                            println!("phone: {} \n department: {:?}", pn.number, department);
                        }
                    });
                }
                None => {}
            
        }
        println!("-----------------------");
    }
}