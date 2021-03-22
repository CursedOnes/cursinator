use cursinator::Op;

use structopt::*;

fn main() {
    env_logger::init();

    std::env::args();
    
    // init game_version
    // search key
    // -> slug name summary is_compatible_game_version
    // install [-a/-b/-r] mod_slug (also recognize project url, file url, file id) [version] (recognize file_id, match_str filename)
    // repl
    // update [-a/-b/-r] mod (recognize even more shit)
    // update-all
    // set-channel

    with_args();
}

fn with_args() {
    let o = Op::from_args();

    cursinator::cmd::main(o)
}
