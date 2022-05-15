const URL_TEMPLATE: &str = "https://videa.hu/kategoriak/film-animacio?sort=0&category=0&page=";

fn main() -> Result<(), reqwest::Error> {
    let page_count = 1;
    for index in 1..page_count + 1 {
        let url = format!("{}{}", URL_TEMPLATE, index);
        let response = reqwest::blocking::get(url)?;
        let text = response.text()?;
        println!("{}", text);
    }
    Ok(())
}
