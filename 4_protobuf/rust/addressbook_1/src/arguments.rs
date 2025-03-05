use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>
}

#[derive(Subcommand)]
pub enum Commands {
    Add(AddArgs),
    List(ListArgs),
}

#[derive(
    clap::ValueEnum, Clone, Default, Debug, 
)]
pub enum PhoneType {
    #[default]
    Undefined,
    Home,
    Mobile,
    Work,
}

#[derive(
    clap::ValueEnum, Clone, Default, Debug, 
)]
pub enum DepType {
    #[default]
    Undefined,
    Hr,
    Cs,
}

#[derive(
    clap::ValueEnum, Clone, Default, Debug, 
)]
pub enum KindType {
    #[default]
    Per,
    Company,
    Cie,
    Person,
}

#[derive(Args)]
pub struct AddArgs {
    #[arg(short, long)]
    pub name: String,

    #[arg(short, long)]
    pub kind: KindType,

    #[arg(short, long)]
    pub email: Option<String>, 

    #[arg(short, long, default_value_t, value_enum)]
    pub dep: DepType,

    #[arg(short, long)]
    pub phone: Option<String>,

    #[arg(short, long, default_value_t, value_enum)]
    pub r#type: PhoneType,
    
}

#[derive(Args)]
pub struct ListArgs {
    #[arg(short, long)]
    pub redact: bool,
}