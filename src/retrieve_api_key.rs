use std::borrow::Cow;

use crate::conf::Conf;
use crate::hard_error;

pub fn cf_api_key(override_api_key: Option<Cow<'static,str>>) -> Cow<'static,str> {
    let integrated_api_key = env!("CURSEFORGE_API_KEY"); // Supply API key at compile time into the build
    //let integrated_api_key = ""; // Build without API key

    //let integrated_api_key = include_str!("../cf_test_key");

    if let Some(key) = override_api_key.filter(|key| !key.is_empty() ) {
        key
    } else if let Some(key) = std::env::var("CURSEFORGE_API_KEY").ok().filter(|key| !key.is_empty() ) {
        Cow::Owned(key)
    } else if !integrated_api_key.is_empty() {
        Cow::Borrowed(integrated_api_key)
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
