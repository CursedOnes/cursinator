use cursinator::Op;

use structopt::*;

#[tokio::main] //TODO use reqwest::blocking in furse or rewrite to async
async fn main() {
    env_logger::init();

    with_args();
}

fn with_args() {
    let o = Op::from_args();

    cursinator::cmd::main(o)
}
