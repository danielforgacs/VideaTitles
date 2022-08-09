mod config;

use crossterm::style::{
    Attribute::{Bold, Reset},
    Color, SetForegroundColor,
};
use std::io::Read;
use dotenv;

const VERSION: &str = "2022.5.17";
const MAX_PAGES: u16 = 250;
const URL_TEMPLATE: &str = "https://videa.hu/kategoriak/film-animacio?sort=0&category=0&page=";
const TITLE_REGEX_PATTERN: &str = r#"<div class="panel-video-title"><a href="(.*)" title=".*">(.*)</a></div>"#;
const YEAR_REGEX_PATTERN: &str = r"(\d{4})";
const MAX_UTF8: u32 = 800;
const BLACKLIST_FILE_NAME: &str = "videablacklist.txt";
const MAX_BAD_CHAR_COUNT: u8 = 5;
const SIMILAR_MATCH_CHAR_COUNT: u8 = 8;
const YEAR_MIN: u16 = 1900;
const YEAR_MAX: u16 = 2035;

type MyResult<T> = Result<T, Box<dyn std::error::Error>>;

#[derive(Debug)]
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

    fn contains_year(&self) -> bool {
        let re = regex::Regex::new(YEAR_REGEX_PATTERN).unwrap();
        match re.captures(&self.title) {
            Some(cap) => {
                let year = match &cap[1].parse::<u16>() {
                    Ok(year) => *year,
                    Err(_) => 0,
                };
                (YEAR_MIN..=YEAR_MAX).contains(&year)
            }
            None => false,
        }
    }
}

impl std::fmt::Display for Movie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<70}{}", self.title, self.url)
    }
}

fn main() -> MyResult<()> {
    dotenv::dotenv().ok();
    let matches = clap::Command::new("vidatitles")
        .about(VERSION)
        .arg(clap::Arg::new("pagecount").default_value("1"))
        .arg(
            clap::Arg::new("pageoffset")
                .short('o')
                .long("offset")
                .default_value("0"),
        )
        .get_matches();
    let page_count = matches
        .value_of("pagecount")
        .unwrap()
        .parse::<u16>()
        .unwrap();
    let page_offset = matches
        .value_of("pageoffset")
        .unwrap()
        .parse::<u16>()
        .unwrap();

    if !(1..=MAX_PAGES).contains(&page_count) {
        println!("Page count must be in range: 1 - {}.", MAX_PAGES);
        return Ok(());
    }

    let re = regex::Regex::new(TITLE_REGEX_PATTERN).unwrap();
    let mut pages: Vec<String> = Vec::new();

    for index in 1..page_count + 1 {
        let url = format!("{}{}", URL_TEMPLATE, index + page_offset);
        let response = reqwest::blocking::get(url)?;
        pages.push(response.text()?);
    }

    let blacklist = read_or_create_blacklist()?;
    let mut movies: Vec<Movie> = vec![];
    for cap in re.captures_iter(&pages.join("\n")) {
        let movie = Movie::from_capture(cap);
        if contains_out_of_range_char(&movie.title) {
            eprintln!("{:<25}{}", "bad char:", movie);
            continue;
        }
        if found_in_blacklist(&movie.title, &blacklist) {
            eprintln!("{:<25}{}", "blacklisted:", movie);
            continue;
        }
        movies.push(movie);
    }

    dbg!(&movies);

    movies.sort_by_key(|m| m.title.clone());

    println!();

    let mut previous_movie = String::from("");
    for movie in movies {
        if is_similar_title(&movie.title, &previous_movie) {
            print!("{}", SetForegroundColor(Color::DarkGrey));
        } else if movie.contains_year() {
            print!("{}", SetForegroundColor(Color::Green));
            print!("{}", Bold);
        };
        print!("{}", movie);
        println!("{}", Reset);
        previous_movie = movie.title.clone();
    }

    Ok(())
}

fn is_similar_title(t0: &str, t1: &str) -> bool {
    let mut match_count = 0;
    let char_count = if t0.len() < t1.len() {
        t0.len()
    } else {
        t1.len()
    };
    for i in 0..char_count {
        if t0.chars().nth(i) == t1.chars().nth(i) {
            match_count += 1;
        }
        if match_count == SIMILAR_MATCH_CHAR_COUNT {
            return true;
        }
    }
    false
}

fn contains_out_of_range_char(title: &str) -> bool {
    let mut bad_char_count = 0;
    for letter in title.chars() {
        let letter_b = letter as u32;
        if letter_b > MAX_UTF8 {
            bad_char_count += 1;
            if bad_char_count > MAX_BAD_CHAR_COUNT {
                return true;
            }
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
        println!(
            "Creating empty blacklist: {}",
            black_list_path.to_str().unwrap()
        );
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
            ("К", false),
            ("КК", false),
            ("ККК", false),
            ("К     КК", false),
            ("КККК", false),
            ("ККККК", false),
            ("КККК      К", false),
            ("КККККК", true),
            ("КККК      К   К ", true),
        ];
        for (title, expected) in cases {
            assert_eq!(contains_out_of_range_char(title), expected);
        }
    }

    #[test]
    fn similar_char_count() {
        assert!(!is_similar_title("12345", "12345"));
        assert!(!is_similar_title("1234567", "1234567"));
        assert!(is_similar_title("12345678", "12345678"));
        assert!(is_similar_title("12345678", "12345678"));
        assert!(!is_similar_title("   !", "   aj"));
    }

    #[test]
    fn finding_year_in_titles() {
        assert!(!Movie {
            title: "laksdfhj".to_string(),
            url: "".to_string(),
        }.contains_year());
        assert!(!Movie {
            title: "laks1234dfhj".to_string(),
            url: "".to_string(),
        }.contains_year());
        assert!(!Movie {
            title: "laks1234 0000 dfhj".to_string(),
            url: "".to_string(),
        }.contains_year());
        assert!(Movie {
            title: "laks123 2000 dfhj".to_string(),
            url: "".to_string(),
        }.contains_year());
        assert!(Movie {
            title: "laks123 aksdj 233 2000".to_string(),
            url: "".to_string(),
        }.contains_year());
        assert!(Movie {
            title: "laks123 aksdj 233 (2002)".to_string(),
            url: "".to_string(),
        }.contains_year());
        assert!(Movie {
            title: "laks123 aksdj 233 (200) .2005.".to_string(),
            url: "".to_string(),
        }.contains_year());
        assert!(Movie {
            title: "a2000b".to_string(),
            url: "".to_string(),
        }.contains_year());
    }
}
