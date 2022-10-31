use std::borrow::Cow;

use crate::conf::Conf;
use crate::hard_error;

pub fn cf_api_key(override_api_key: Option<Cow<'static,str>>) -> Cow<'static,str> {
    let integrated_api_key = Some(env!("CURSEFORGE_API_KEY")); // Supply API key at compile time into the build
    //let integrated_api_key = None; // Build without API key

    //let integrated_api_key = Some(include_str!("../cf_test_key"));

    if let Some(v) = override_api_key {
        v
    } else if let Ok(v) = std::env::var("CURSEFORGE_API_KEY") {
        Cow::Owned(v)
    } else if let Some(v) = integrated_api_key {
        Cow::Borrowed(v)
    } else {
        use termion::*;

        eprintln!(
            "{}{}This build of {} doesn't contain a CurseForge API key and the API key wasn't supplied at runtime

The API key must either:
\t- supplied at compile time for the build via CURSEFORGE_API_KEY environment variable
\t- supplied at runtime via CURSEFORGE_API_KEY environment variable
\t- supplied in the repo.json conf.override_api_key{}{}",
            color::Fg(color::LightRed),style::Bold,
            env!("CARGO_PKG_NAME"),
            style::Reset,color::Fg(color::Reset)
        );
        hard_error!("API key cannot be retrieved");
    }
}
