pub mod util;
pub mod addon;
pub mod api;
pub mod conf;
pub mod print;
pub mod op;
pub mod cmd;
pub mod retrieve_api_key;
// what if mods are bigger than 4 GiB?
//#[cfg(not(target_pointer_width = "64"))]
//compile_error!("only 64-bit pointer arch supported");

use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "cursinator", about = "Download and manage CurseForge addons")]
pub struct Op {
    /// Path to repo json
    #[arg(short,long,default_value="repo.json")]
    pub conf: PathBuf,
    /// spam stderr
    #[arg(short,long)]
    pub verbose: bool,
    /// Just print what would happen
    #[arg(short='n',long)]
    pub noop: bool,
    /// No queries to online api
    #[arg(long)]
    pub offline: bool, //TODO bork all API when offline mode
    #[command(subcommand)]
    pub cmd: OpCmd,
}
#[derive(Subcommand,Clone)]
pub enum OpCmd {
    /// Initialize local mod repo
    #[command()]
    Init {
        #[arg(short='g',long,help="gv")]
        game_version: Option<String>,
        #[arg(short='G',long,help="gv")]
        game_version_regex: Option<String>,
    },
    /// Search online for addon
    #[command()]
    Search {
        /// # results (0=dynamic)
        #[arg(short='p',long="page-size",default_value="0")]
        page_size: u32,
        /// page index
        #[arg(short='n',long="page-n",default_value="0")]
        page_n: u32,
        #[arg(help="addon")]
        addon: String,
    },
    /// Install addon
    #[command()]
    Install {
        /// ignored if explicit version given
        #[arg(short,long)]
        alpha: bool,
        /// ignored if explicit version given
        #[arg(short,long)]
        beta: bool,
        /// ignored if explicit version given
        #[arg(short,long)]
        release: bool,
        /// Install even if incompatibility occurs
        #[arg(short='f',long)]
        force: bool,
        /// version blacklist
        #[arg(short='x',long="version-blacklist")]
        version_blacklist: Option<String>,
        // Addon slug or id, with optional version specified, must be non-ambiguous
        #[arg()]
        addons: Vec<String>,
    },
    /// Update addon
    #[command()]
    Update {
        /// ignored if explicit version given, will override addon's channel, but won't change addon's channel
        #[arg(short,long)]
        alpha: bool,
        /// ignored if explicit version given, will override addon's channel, but won't change addon's channel
        #[arg(short,long)]
        beta: bool,
        /// ignored if explicit version given, will override addon's channel, but won't change addon's channel
        #[arg(short,long)]
        release: bool,
        /// Update even if incompatibility occurs
        #[arg(short='f',long)]
        force: bool,
        /// Allow downgrade (explicit version install always allows downgrade)
        #[arg(short='d',long="allow-downgrade")]
        allow_downgrade: bool,
        /// Match addon slug, id or installed filename, must be non-ambiguous
        #[arg()]
        addon: String, // if "list", do list -u
        /// Optional: Install specific version of addon, implies allow_downgrade
        #[arg()]
        file: Option<String>,
    },
    /// Set/Get release mode channel for addon
    /// 
    /// Example:
    /// cursinator channel iron-chests    | shows current channel
    /// cursinator channel iron-chests ba | set channel to ba
    /// 
    /// Channel is denoted by the letters
    /// 
    /// Channel examples:
    /// r = latest release
    /// b = latest beta-ish (beta/release)
    /// a = latest alpha-ish
    /// rb = latest release, if no release available, fallback to latest beta-ish
    /// ba = latest beta-ish, fallback to alpha
    /// rba (default) = latest release, fallback to beta or alpha
    /// ra = release, fallback to alpha
    /// Order doesn't matter: abr = arb = bar = rba
    #[command(name = "channel", verbatim_doc_comment)]
    Channel {
        /// Match addon slug, id or installed filename which channel should be changed, must be non-ambiguous
        #[arg()]
        addon: String,
        /// Show channel if not set, set channel if set
        #[arg()]
        value: Option<String>,
    },
    /// List installed addons
    #[command()]
    List {
        
    },
    /// List available updates or addon versions
    /// 
    /// Use -a/-b/-r to control which release types should be shown, if none of the three set, the r/b/a of addon's channel are implied
    #[command()]
    Updates{
        /// Show alphas, betas and releases (if -b or -r is also set and betas/releases are available, alphas/betas won't be shown)
        #[arg(short,long)]
        alpha: bool,
        /// show only betas and releases if betas or releases are available (if -r and releases available, only releases will be shown)
        #[arg(short,long)]
        beta: bool,
        /// show only releases if releases are available
        #[arg(short,long)]
        release: bool,
        /// Also list addons without updates and don't omit versions
        #[arg(short='s',long="show-all")]
        show_all: bool,
        /// Also show older versions when listing versions of addon
        #[arg(short='o',long="older")]
        older: bool,
        /// Match addon slug, id or installed filename for which updates should be shown, must be non-ambiguous
        #[arg()]
        addon: Option<String>, //with addon just list available versions, this would fallback to list version of not installed addons (with query)
    },
    /// Update all addons
    #[command(name = "update-all")]
    UpdateAll {
        /// TODO
        #[arg(short,long)]
        alpha: bool,
        /// TODO
        #[arg(short,long)]
        beta: bool,
        /// TODO
        #[arg(short,long)]
        release: bool,
    },
    /// Download all addons if not already downloaded or invalid
    #[command(name = "download-all")]
    DownloadAll {
        /// Only fill cache
        #[arg(short,long)]
        cache_only: bool,
    },
    /// Remove addon. Use purge to also remove metadata/information/settings of the addon
    #[command()]
    Remove {
        /// Remove addon even if other addons depend on this addon
        #[arg(short='f',long)]
        force: bool,
        /// Match addon slug, id or installed filename which should be removed, must be non-ambiguous
        #[arg()]
        addon: String,
    },
    /// Remove addons which were installed as dependency
    #[command(name = "autoremove")]
    AutoRemove {
        /// Also purge information about addons removed in this operation
        #[arg(short='p',long)]
        purge: bool,
    },
    /// Remove and purge information about installed addons, can also be done on removed addons
    #[command()]
    Purge {
        /// Remove addon even if other addons depend on this addon
        #[arg(short='f',long)]
        force: bool,
        /// Only purge if already removed
        #[arg(short='c',long)]
        cleanup_only: bool,
        /// Match addon slug, id or installed filename which should be purged, must be non-ambiguous
        #[arg()]
        addon: String,
    },
    /// Purge information over all already removed addons
    #[command(name = "purge-removed")]
    PurgeRemoved {
        
    },
    // /// Rename addon to .disabled
    // #[command()]
    // Disable { //TODO check for dependent addons
    //     /// Disable addon even if other addons depend on this addon
    //     #[arg(short='f',long)]
    //     force: bool,
    //     /// Also disable addons that depend on this addons recursively
    //     #[arg(long="disable-depending")]
    //     disable_depending: bool,
    //     /// TODO
    //     #[arg()]
    //     addon: String,
    // },
    // /// Enable .disabled addon
    // #[command()]
    // Enable {
    //     /// TODO
    //     #[arg()]
    //     addon: String,
    // },
    /// Addon setting. Not all options exposed yet, refer repo.json
    #[command()]
    Aset{ //TODO move update_opt to separate option
        /// Match addon slug, id or installed filename, must be non-ambiguous
        #[arg()]
        addon: String,
        /// Show/Set specific setting, else list settings
        #[arg()]
        key: Option<String>,
        /// Set setting, else show setting
        #[arg()]
        value: Option<String>,
    },
    /// Repo setting. Not all options exposed yet, refer repo.json
    #[command()]
    Rset{
        /// Show/Set specific setting, else list settings
        #[arg()]
        key: Option<String>,
        /// Set setting, else show setting
        #[arg()]
        value: Option<String>,
    },
    /// Generate CF manifest.json from template
    #[command(name = "gen-cf-manifest")]
    GenCfManifest {
        /// Input template
        #[arg()]
        input: PathBuf,
        /// Output manifest.json
        #[arg()]
        output: PathBuf,
    },
}
