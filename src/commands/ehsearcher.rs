use std::collections::HashSet;
use std::fs;

use nom::{
    bytes::complete::{is_a, tag, take_till1},
    IResult,
};
use reqwest::{Client, Error};
use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};
use serde_json::json;

struct ApiData {
    gallery_id: i32,
    gallery_token: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct GalleryWrapper {
    gmetadata: Vec<Gallery>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Gallery {
    pub gid: u64,
    pub token: String,
    pub title: String,
    pub title_jpn: Option<String>,
    pub thumb: String,
    pub posted: String,
    pub filecount: String,
    pub rating: String,
    pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct TagTypes {
    warning_tags: Vec<String>,
    bad_tags: Vec<String>,
    illegal_tags: Vec<String>,
}

fn parse_actual_tag(input: &str) -> IResult<&str, &str> {
    let (input, _) = is_a("aefhlmort")(input)?;
    let (input, _) = tag(":")(input)?;

    Ok((input, input))
}

fn parse_api_data(input: &str) -> IResult<&str, ApiData> {
    let (input, _) = tag("https://e-hentai.org/g/")(input)?;
    let (input, g_id) = take_till1(|c| c == '/')(input)?;
    let (input, _) = tag("/")(input)?;
    let (input, g_token) = take_till1(|c| c == '/')(input)?;

    Ok((
        input,
        ApiData {
            gallery_id: g_id.parse().unwrap(),
            gallery_token: g_token.to_owned(),
        },
    ))
}

fn clean_tags(tags: Vec<&str>) -> Vec<&str> {
    tags.iter()
        .filter(|t| t.starts_with("male:") || t.starts_with("female:") || t.starts_with("other:"))
        .map(|t| {
            let (_, tag) = parse_actual_tag(t).unwrap_or(("", ""));
            tag
        })
        .collect()
}

pub async fn get_top_results(
    query: &str,
    manga_only: bool,
    restrictive: bool,
) -> Result<Vec<Gallery>, Error> {
    let client = Client::new();
    let query = query.replace(" ", "+");
    let body = match manga_only {
        true => {
            client
                .get(format!(
                    "https://e-hentai.org/?f_cats=1017&f_search={query}"
                ))
                .send()
                .await?
                .text()
                .await?
        }
        false => {
            client
                .get(format!("https://e-hentai.org/?f_search={query}"))
                .send()
                .await?
                .text()
                .await?
        }
    };

    let mut galleries = vec![];

    let body = Html::parse_document(&body);
    let row_selector = Selector::parse(".gl3c").expect("Could not find table row");
    let link_selector = Selector::parse("a").expect("Could not find link");
    let tag_div_selector = Selector::parse("div").expect("Could not find tags");

    let data = fs::read_to_string("../tags.json").expect("could not read file");
    let tag_types: TagTypes = serde_json::from_str(&data).expect("data was not formatted properly");

    let illegal_tags: HashSet<_> = tag_types.illegal_tags.iter().collect();
    let bad_tags: HashSet<_> = tag_types.bad_tags.iter().collect();

    'rows: for row in body.select(&row_selector) {
        let link_element = row.select(&link_selector).next().unwrap();
        let link = link_element.value().attr("href").unwrap();

        let first_tags = link_element
            .select(&tag_div_selector)
            .next()
            .unwrap()
            .text()
            .map(|t| {
                let (_, tag) = parse_actual_tag(t).unwrap();
                tag
            })
            .collect::<Vec<_>>();

        for tag in first_tags {
            if illegal_tags.contains(&tag.to_owned()) {
                continue 'rows;
            }
            if restrictive && bad_tags.contains(&tag.to_owned()) {
                continue 'rows;
            }
        }

        let (_, api_data) = parse_api_data(link).unwrap();

        let gallery_wrapper = client
            .post("https://api.e-hentai.org/api.php")
            .json(&json!({
                "method": "gdata",
                "gidlist": [
                    [api_data.gallery_id, api_data.gallery_token]
                ],
                 "namespace": 1
            }))
            .send()
            .await?
            .json::<GalleryWrapper>()
            .await?;

        let gallery = gallery_wrapper.gmetadata.first().unwrap();
        let gallery_tags = clean_tags(gallery.tags.iter().map(|t| t.as_str()).collect());

        for tag in gallery_tags {
            if illegal_tags.contains(&tag.to_owned()) {
                continue 'rows;
            }
            if restrictive && bad_tags.contains(&tag.to_owned()) {
                continue 'rows;
            }
        }

        galleries.push(gallery.clone());
    }

    Ok(galleries)
}
