use anyhow::anyhow;

use crate::util::match_str::Match;

pub fn unwrap_match<T>(r: Result<Match<T>,Vec<Match<T>>>) -> anyhow::Result<Match<T>> {
    match r {
        Ok(r) => Ok(r),
        Err(e) if e.is_empty() => Err(anyhow!("No match for installed addon")),
        Err(e) => {
            let mut error_message = "Ambiguous matches for installed addon".to_owned();
            for m in e {
                error_message += &m.fmt_error("\n");
            }
            Err(anyhow!(error_message))
        }
    }
}
