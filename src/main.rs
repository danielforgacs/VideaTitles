const URL_TEMPLATE: &str = "https://videa.hu/kategoriak/film-animacio?sort=0&category=0&page=";

fn main() -> Result<(), reqwest::Error> {
    let page_count = 1;
    let title_regex_pattern = r#"<div class="panel-video-title"><a href="(.*)" title=".*">(.*)</a></div>"#;
    // let title_regex_pattern = r#"panel-video-title"#;
    let re = regex::Regex::new(title_regex_pattern).unwrap();
    for index in 1..page_count + 1 {
        let url = format!("{}{}", URL_TEMPLATE, index);
        let response = reqwest::blocking::get(url)?;
        let text = response.text()?;
        for cap in re.captures_iter(&text) {
            let movie_url = &cap[1];
            let movie_title = &cap[2];
            println!("{:<70}{}", movie_title, movie_url);
        }
    }
    Ok(())
}
