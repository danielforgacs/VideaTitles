mod config;
#[macro_use]
extern crate diesel;

mod schema;
use schema::movie::dsl::*;
use schema::movie;

use crossterm::style::{
    Attribute::{Bold, Reset},
    Color, SetForegroundColor,
};
use std::io::Read;
use dotenv;
use diesel::prelude::*;
use diesel::pg::PgConnection;

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

#[derive(Queryable, Debug)]
struct Movie {
    id: i32,
    title: String,
    url: String,
}

#[derive(Insertable)]
#[table_name="movie"]
struct NewMovie {
    title: String,
    url: String,
}


#[derive(Debug)]
struct SimpleMovie {
    title: String,
    url: String,
}

impl NewMovie {
    fn new(new_title: String, new_url: String) -> Self {
        Self {
            title: new_title,
            url: new_url,
        }
    }
}

impl SimpleMovie {
    fn from_capture(cap: regex::Captures) -> Self {
        SimpleMovie {
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

impl std::fmt::Display for SimpleMovie {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:<70}{}", self.title, self.url)
    }
}

fn establish_db_conn() -> PgConnection {
    let db_url = std::env::var("DATABASE_URL").expect("DATABASE_URL env var must be set.");
    PgConnection::establish(&db_url).expect("Can't connect to the database.")
}

fn insert_new_movie(new_movie: NewMovie) {
    let db_conn = establish_db_conn();
    diesel::insert_into(movie::table)
        .values(&new_movie)
        .get_result::<Movie>(&db_conn)
        .expect("Can't insert new movie.");
}

fn insert_new_movies(new_movies: Vec<NewMovie>) {
    let db_conn = establish_db_conn();
    diesel::insert_into(movie::table)
        .values(&new_movies)
        .get_result::<Movie>(&db_conn)
        .expect("Can't insert new movies.");
}

fn main() {
    let config = config::get_config();
}

fn main_old() -> MyResult<()> {
    dotenv::dotenv().ok();

    //
    let db_conn = establish_db_conn();
    let db_movies = movie
        .load::<Movie>(&db_conn);
    dbg!(db_movies);

    let new_movie = NewMovie {
        title: "NEW_TITLE".to_string(),
        url: "NEW_URL".to_string(),
    };

    diesel::insert_into(movie::table)
        .values(&new_movie)
        .get_result::<Movie>(&db_conn)
        .expect("Error saving new movie");

    //

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
        let request_url = format!("{}{}", URL_TEMPLATE, index + page_offset);
        let response = reqwest::blocking::get(request_url)?;
        pages.push(response.text()?);
    }

    let blacklist = read_or_create_blacklist()?;
    let mut simple_movies: Vec<SimpleMovie> = vec![];
    let mut new_movies: Vec<NewMovie> = vec![];
    for cap in re.captures_iter(&pages.join("\n")) {
        let simple_movie = SimpleMovie::from_capture(cap);
        let new_movie = NewMovie::new(
            simple_movie.title.clone(),
            simple_movie.url.clone()
        );
        new_movies.push(
            NewMovie::new(
                simple_movie.title.clone(),
                simple_movie.url.clone()
            )
        );

        // insert_new_movie(new_movie);
        if contains_out_of_range_char(&simple_movie.title) {
            eprintln!("{:<25}{}", "bad char:", simple_movie);
            continue;
        }
        if found_in_blacklist(&simple_movie.title, &blacklist) {
            eprintln!("{:<25}{}", "blacklisted:", simple_movie);
            continue;
        }
        simple_movies.push(simple_movie);
    }
    insert_new_movies(new_movies);



    simple_movies.sort_by_key(|m| m.title.clone());

    println!();

    let mut previous_movie = String::from("");
    for simple_movie in simple_movies {
        if is_similar_title(&simple_movie.title, &previous_movie) {
            print!("{}", SetForegroundColor(Color::DarkGrey));
        } else if simple_movie.contains_year() {
            print!("{}", SetForegroundColor(Color::Green));
            print!("{}", Bold);
        };
        print!("{}", simple_movie);
        println!("{}", Reset);
        previous_movie = simple_movie.title.clone();
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

fn contains_out_of_range_char(text: &str) -> bool {
    let mut bad_char_count = 0;
    for letter in text.chars() {
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

fn found_in_blacklist(text: &str, blacklist: &str) -> bool {
    for phrase in blacklist.lines() {
        if text.contains(phrase) {
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
        for (text, expected) in cases {
            assert_eq!(contains_out_of_range_char(text), expected);
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
        assert!(!SimpleMovie {
            title: "laksdfhj".to_string(),
            url: "".to_string(),
        }.contains_year());
        assert!(!SimpleMovie {
            title: "laks1234dfhj".to_string(),
            url: "".to_string(),
        }.contains_year());
        assert!(!SimpleMovie {
            title: "laks1234 0000 dfhj".to_string(),
            url: "".to_string(),
        }.contains_year());
        assert!(SimpleMovie {
            title: "laks123 2000 dfhj".to_string(),
            url: "".to_string(),
        }.contains_year());
        assert!(SimpleMovie {
            title: "laks123 aksdj 233 2000".to_string(),
            url: "".to_string(),
        }.contains_year());
        assert!(SimpleMovie {
            title: "laks123 aksdj 233 (2002)".to_string(),
            url: "".to_string(),
        }.contains_year());
        assert!(SimpleMovie {
            title: "laks123 aksdj 233 (200) .2005.".to_string(),
            url: "".to_string(),
        }.contains_year());
        assert!(SimpleMovie {
            title: "a2000b".to_string(),
            url: "".to_string(),
        }.contains_year());
    }
}
