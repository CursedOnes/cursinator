pub mod util;
pub mod addon;
pub mod api;
pub mod conf;
pub mod print;
pub mod op;
pub mod cmd;
// what if mods are bigger than 4 GiB?
//#[cfg(not(target_pointer_width = "64"))]
//compile_error!("only 64-bit pointer arch supported");

use std::path::PathBuf;

use structopt::*;

#[derive(StructOpt)]
#[structopt(name = "cursinator", about = "Download and manage CurseForge addons")]
pub struct Op {
    #[structopt(short,long,default_value="repo.json",help="path to repo json")]
    pub conf: PathBuf,
    #[structopt(short,long,help="spam stderr")]
    pub verbose: bool,
    #[structopt(short="n",long,help="Just print what would happen")]
    pub noop: bool,
    #[structopt(long,help="No queries to online api")]
    pub offline: bool, //TODO bork all API when offline mode
    #[structopt(subcommand)]
    pub cmd: OpCmd,
}
#[derive(StructOpt,Clone)]
pub enum OpCmd {
    #[structopt(about = "Initialize local mod repo")]
    Init {
        #[structopt(short="g",long,help="gv")]
        game_version: Option<String>,
        #[structopt(short="G",long,help="gv")]
        game_version_regex: Option<String>,
    },
    #[structopt(about = "Search online for addon")]
    Search {
        #[structopt(short="p",long="page-size",default_value="0",help="# results (0=dynamic)")]
        page_size: u32,
        #[structopt(short="n",long="page-n",default_value="0",help="page index")]
        page_n: u32,
        #[structopt(help="addon")]
        addon: String,
    },
    #[structopt(about = "Install addon")]
    Install {
        #[structopt(short,long,help="ignored if explicit version given")]
        alpha: bool,
        #[structopt(short,long,help="ignored if explicit version given")]
        beta: bool,
        #[structopt(short,long,help="ignored if explicit version given")]
        release: bool,
        #[structopt(short="f",long,help="Install even if incompatibility occurs")]
        force: bool,
        #[structopt(short="x",long="version-blacklist",help="version blacklist")]
        version_blacklist: Option<String>,
        #[structopt(help="addon slug or id")]
        slug: String,
        #[structopt(help="version")]
        file: Option<String>,
    },
    #[structopt(about = "Update addon")]
    Update {
        #[structopt(short,long,help="ignored if explicit version given, will override addon's channel, but won't change addon's channel")]
        alpha: bool,
        #[structopt(short,long,help="ignored if explicit version given, will override addon's channel, but won't change addon's channel")]
        beta: bool,
        #[structopt(short,long,help="ignored if explicit version given, will override addon's channel, but won't change addon's channel")]
        release: bool,
        #[structopt(short="f",long,help="Update even if incompatibility occurs")]
        force: bool,
        #[structopt(short="d",long="allow-downgrade",help="Allow downgrade (explicit version install always allows downgrade)")]
        allow_downgrade: bool,
        #[structopt(help="slug")]
        addon: String, // if "list", do list -u
        #[structopt(help="Explicit version")]
        file: Option<String>,
    },
    #[structopt(name = "channel", about = "Set/Get release mode channel for addon", help = "Set/Get release mode channel for addon\n\nExample:\ncursinator channel iron-chests    | shows current channel\ncursinator channel iron-chests ba | set channel to ba\n\nChannel is denoted by the letters\n\nChannel examples:\nr = latest release\nb = latest beta-ish (beta/release)\na = latest alpha-ish\nrb = latest release, if no release available, fallback to latest beta-ish\nba = latest beta-ish, fallback to alpha\nrba (default) = latest release, fallback to beta or alpha\nra = release, fallback to alpha\nOrder doesn't matter: abr = arb = bar = rba")]
    Channel {
        #[structopt(help="addon")]
        addon: String,
        #[structopt(help="value or show channel")]
        value: Option<String>,
    },
    #[structopt(about = "List installed addons")]
    List {
        
    },
    #[structopt(about = "List available updates or addon versions")]
    Updates{
        #[structopt(short,long,help="show alphas (except -b/-r and betas/releases available)")]
        alpha: bool,
        #[structopt(short,long,help="show only betas and releases if betas are available (if -r and releases available, only releases will be shown)")]
        beta: bool,
        #[structopt(short,long,help="show only releases if releases are available")]
        release: bool,
        #[structopt(short="s",long="show-all",help="also list addons without updates and don't omit versions")]
        show_all: bool,
        #[structopt(short="o",long="older",help="show older versions when listing versions of addon")]
        older: bool,
        #[structopt(help="addon")]
        addon: Option<String>, //with addon just list available versions, this would fallback to list version of not installed addons (with query)
    },
    #[structopt(name = "update-all",about="Update all addons")]
    UpdateAll {
        #[structopt(short,long)]
        alpha: bool,
        #[structopt(short,long)]
        beta: bool,
        #[structopt(short,long)]
        release: bool,
    },
    #[structopt(about="Remove addon")]
    Remove {
        #[structopt(short="f",long,help="Remove even if dependents")]
        force: bool,
        #[structopt(help="addon")]
        addon: String,
    },
    #[structopt(name = "autoremove",about="Remove addons which were installed as dependency")]
    AutoRemove {
        #[structopt(short="p",long,help="Also purge information about addons removed in this operation")]
        purge: bool,
    },
    #[structopt(about="Purge information about installed addons, can also be done on removed addons")]
    Purge {
        #[structopt(short="f",long,help="Remove even if dependents")]
        force: bool,
        #[structopt(short="c",long,help="Only purge if already removed")]
        cleanup_only: bool,
        #[structopt(help="addon")]
        addon: String,
    },
    #[structopt(name = "purge-removed",about="Purge information over all already removed addons")]
    PurgeRemoved {
        
    },
    #[structopt(about="Enable .disabled addon")]
    Disable { //TODO check for dependent addons
        #[structopt(help="addon")]
        addon: String,
    },
    #[structopt(about="rename addon to .disabled")]
    Enable {
        #[structopt(help="addon")]
        addon: String,
    },
    #[structopt(about="Addon setting")]
    Aset{ //TODO move update_opt to separae option
        #[structopt(help="addon")]
        addon: String,
        #[structopt(help="Show/Set specific setting, else list settings")]
        key: Option<String>,
        #[structopt(help="Set setting, else show setting")]
        value: Option<String>,
    },
    #[structopt(about="Repo setting")]
    Rset{
        #[structopt(help="Show/Set specific setting, else list settings")]
        key: Option<String>,
        #[structopt(help="Set setting, else show setting")]
        value: Option<String>,
    },
}
