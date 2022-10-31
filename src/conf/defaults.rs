pub fn default_api_headers() -> Vec<(String,String)> {
    vec![]
}
pub fn default_api_domain() -> String {
    "https://api.curseforge.com/v1".to_owned()
}
pub fn default_url_txt() -> bool {
    true
}
pub fn default_addon_mtime() -> bool {
    true
}
pub fn default_soft_retries() -> u32 {
    4
}
