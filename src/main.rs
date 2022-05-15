use std::io::Read;

const MAX_PAGES: u16 = 250;
const URL_TEMPLATE: &str = "https://videa.hu/kategoriak/film-animacio?sort=0&category=0&page=";
const TITLE_REGEX_PATTERN: &str = r#"<div class="panel-video-title"><a href="(.*)" title=".*">(.*)</a></div>"#;
const MAX_UTF8: u32 = 800;

struct Movie {
    title: String,
    url: String,
}

impl Movie {
    fn from_capture(cap: regex::Captures) -> Self {
        Movie {
            title: cap[2].to_owned(),
            url: cap[1].to_owned(),
        }
    }
}

impl std::fmt::Display for Movie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<70}{}", self.title, self.url)
    }
}

fn main() -> Result<(), reqwest::Error> {
    let mut file = std::fs::File::open("blacklist.txt").unwrap();
    let mut blacklist = String::new();
    file.read_to_string(&mut blacklist).unwrap();
    let matches = clap::Command::new("vidatitles")
        .arg(clap::Arg::new("pagecount").default_value("1"))
        .get_matches();
    let page_count = matches.value_of("pagecount").unwrap().parse::<u16>().unwrap();

    if page_count < 1 || page_count > MAX_PAGES {
        println!("Page count must be in range: 1 - {}.", MAX_PAGES);
        return Ok(());
    }

    let re = regex::Regex::new(TITLE_REGEX_PATTERN).unwrap();
    let mut movies: Vec<Movie> = vec![];

    for index in 1..page_count + 1 {
        let url = format!("{}{}", URL_TEMPLATE, index);
        let response = reqwest::blocking::get(url)?;
        let text = response.text()?;

        for cap in re.captures_iter(&text) {
            let movie = Movie::from_capture(cap);
            if contains_out_of_range_char(&movie.title) {
                continue;
            }
            for phrase in blacklist.lines() {
                if movie.title.contains(phrase) {
                    continue;
                }
            }
            movies.push(movie);
        }
    }

    movies.sort_by_key(|m| m.title.clone());

    println!("");
    for movie in movies {
        println!("{}", movie);
    }

    Ok(())
}

fn contains_out_of_range_char(title: &str) -> bool {
    for letter in title.chars() {
        if letter as u32 > MAX_UTF8 {
            eprintln!(r#"skipping on bad char: {:>6} (as u32): "{}" - {}"#, letter as u32, letter, title);
            return true;
        }
    }
    false
}
