use cursinator::Op;

use structopt::*;

fn main() {
    env_logger::init();

    with_args();
}

fn with_args() {
    let o = Op::from_args();

    cursinator::cmd::main(o)
}
