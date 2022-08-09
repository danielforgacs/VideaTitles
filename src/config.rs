pub struct Config {
    url_prefix: String,
}

impl Config {
    pub fn new() -> Self {
        Self {
            url_prefix: "https://videa.hu/videok/film-animacio/".to_string(),
        }
    }

    pub fn url_prefix(&self) -> &str {
        &self.url_prefix
    }
}
