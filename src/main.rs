use std::io::Read;

const MAX_PAGES: u16 = 250;
const URL_TEMPLATE: &str = "https://videa.hu/kategoriak/film-animacio?sort=0&category=0&page=";
const TITLE_REGEX_PATTERN: &str = r#"<div class="panel-video-title"><a href="(.*)" title=".*">(.*)</a></div>"#;
const MAX_UTF8: u32 = 800;
const BLACKLIST_FILE_NAME: &str = ".videablacklist.txt";
const ALLOWED_CHARS: [char; 2] = [
    '–',
    '‼',
];

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

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

fn main() -> MyResult<()> {
    let matches = clap::Command::new("vidatitles")
        .arg(clap::Arg::new("pagecount").default_value("1"))
        .get_matches();
    let page_count = matches.value_of("pagecount").unwrap().parse::<u16>().unwrap();

    if page_count < 1 || page_count > MAX_PAGES {
        println!("Page count must be in range: 1 - {}.", MAX_PAGES);
        return Ok(());
    }

    let re = regex::Regex::new(TITLE_REGEX_PATTERN).unwrap();
    let mut pages: Vec<String> = Vec::new();

    for index in 1..page_count + 1 {
        let url = format!("{}{}", URL_TEMPLATE, index);
        let response = reqwest::blocking::get(url)?;
        pages.push(response.text()?);
    }

    let blacklist = read_or_create_blacklist()?;
    let mut movies: Vec<Movie> = vec![];
    for cap in re.captures_iter(&pages.join("\n")) {
        let movie = Movie::from_capture(cap);
        if contains_out_of_range_char(&movie.title) {
            continue;
        }
        if found_in_blacklist(&movie.title, &blacklist) {
            continue;
        }
        movies.push(movie);
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
        if letter as u32 > MAX_UTF8 && !ALLOWED_CHARS.contains(&letter) {
            eprintln!(r#"skipping on bad char: {:>6} (as u32): "{}" - {}"#, letter as u32, letter, title);
            return true;
        }
    }
    false
}

fn found_in_blacklist(title: &str, blacklist: &str) -> bool {
    for phrase in blacklist.lines() {
        if title.contains(phrase) {
            return true;
        }
    }
    false
}

fn read_or_create_blacklist() -> MyResult<String> {
    let home_dir = std::env::var("HOME")?;
    let black_list_path = std::path::Path::new(&home_dir);
    let black_list_path = black_list_path.canonicalize()?;
    let black_list_path = black_list_path.join(BLACKLIST_FILE_NAME);

    if !black_list_path.is_file() {
        println!("Creating empty blacklist: {}", black_list_path.to_str().unwrap());
        std::fs::File::create(&black_list_path)?;
    };

    let mut file = std::fs::File::open(black_list_path)?;
    let mut blacklist = String::new();
    file.read_to_string(&mut blacklist).unwrap();
    Ok(blacklist)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn finds_illegal_characters() {
        let cases = [
            ("Каан Урганджъоулу- репортаж", true),
            ("Luke 11:9-13 – How to Get the Holy Spirit!", false),
            ("Tiltott gyümölcs - 304. rész ‼️💭", false),
        ];
        for (title, expected) in cases {
            assert_eq!(contains_out_of_range_char(title), expected);
        }
    }
}
