pub struct Config {
    url_prefix: String,
    page_count: u16,
    page_offset: u16,
}

impl Config {
    pub fn new() -> Self {
        Self {
            url_prefix: "https://videa.hu/videok/film-animacio/".to_string(),
            page_count: 1,
            page_offset: 0,
        }
    }

    pub fn url_prefix(&self) -> &str {
        &self.url_prefix
    }

    pub fn set_page_count(&mut self, page_count: u16) {
        self.page_count = page_count
    }

    pub fn set_page_offset(&mut self, page_offset: u16) {
        self.page_offset = page_offset
    }
}
