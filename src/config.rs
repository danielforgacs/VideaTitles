const VERSION: &str = "2022.5.17";
const MAX_PAGES: u16 = 250;

#[derive(Debug)]
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

    pub fn get_url_prefix(&self) -> &str {
        &self.url_prefix
    }

    pub fn set_page_count(mut self, page_count: u16) -> Self {
        self.page_count = page_count;
        self
    }

    pub fn set_page_offset(mut self, page_offset: u16) -> Self {
        self.page_offset = page_offset;
        self
    }
}

pub fn get_config() -> Result<Config, String> {
    let matches = clap::Command::new("vidatitles")
        .about(VERSION)
        .arg(
            clap::Arg::new("pagecount")
            .default_value("1")
        )
        .arg(
            clap::Arg::new("pageoffset")
                .short('o')
                .long("offset")
                .default_value("0"),
        )
        .get_matches();

    let mut page_count = matches
        .value_of("pagecount")
        .unwrap()
        .parse::<u16>()
        .unwrap();
    let page_offset = matches
        .value_of("pageoffset")
        .unwrap()
        .parse::<u16>()
        .map_err(|_| format!("Page offset must be in range: 0 - {}.", 9999))?;

    if !(1..=MAX_PAGES).contains(&page_count) {
        return Err(
            format!("Page count must be in range: 1 - {}.", MAX_PAGES)
        );
    }

    Ok(
        Config::new()
            .set_page_count(page_count)
            .set_page_offset(page_offset)
    )
}
