pub fn default_headers() -> Vec<(String,String)> {
    vec![]
}
pub fn default_domain() -> String {
    "https://addons-ecs.forgesvc.net".to_owned()
}
pub fn default_url_txt() -> bool {
    true
}
pub fn default_addon_mtime() -> bool {
    true
}
pub fn default_soft_retries() -> usize {
    2
}
