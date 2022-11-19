use clap::Parser;
use cursinator::Op;

fn main() {
    env_logger::init();

    with_args();
}

fn with_args() {
    let o = Op::parse();

    cursinator::cmd::main(o)
}
