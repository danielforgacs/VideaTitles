const PAGE_COUNT: u16 = 1;
const URL_TEMPLATE: &str = "https://videa.hu/kategoriak/film-animacio?sort=0&category=0&page=";
const TITLE_REGEX_PATTERN: &str = r#"<div class="panel-video-title"><a href="(.*)" title=".*">(.*)</a></div>"#;

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
    let re = regex::Regex::new(TITLE_REGEX_PATTERN).unwrap();
    let mut movies: Vec<Movie> = vec![];

    for index in 1..PAGE_COUNT + 1 {
        let url = format!("{}{}", URL_TEMPLATE, index);
        let response = reqwest::blocking::get(url)?;
        let text = response.text()?;

        for cap in re.captures_iter(&text) {
            movies.push(Movie::from_capture(cap));
        }
    }

    movies.sort_by_key(|m| m.title.clone());

    for movie in movies {
        println!("{}", movie);
    }

    Ok(())
}
