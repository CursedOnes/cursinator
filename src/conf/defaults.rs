pub fn default_api_headers() -> Vec<(String,String)> {
    vec![]
}
pub fn default_api_domain() -> String {
    "https://addons-ecs.forgesvc.net/api/v2".to_owned()
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
