use std::str::FromStr;
use std::sync::Arc;

use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::http::Typing;
use serenity::model::prelude::*;
use serenity::prelude::*;

use chrono::{DateTime, NaiveDateTime, Utc};

use super::ehsearcher::get_top_results;

#[command]
pub async fn search(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    let restrictive_flag = args.parse::<String>()?;
    let restrictive = match restrictive_flag.as_str() {
        "-n" => false,
        _ => true,
    };
    let query = args.rest();

    let typing = Typing::start(ctx.http, *msg.channel_id.as_u64())?;

    let galleries = get_top_results(query, true, restrictive).await?;
    if galleries.len() == 0 {
        msg.channel_id.say(&ctx.http, "No galleries found!").await?;
    } else if galleries.len() == 1 {
        let gallery = galleries.first().unwrap();
        let artist = gallery
            .tags
            .iter()
            .map(|t| t.as_str())
            .filter(|t| t.starts_with("artist:"))
            .map(|t| t.get(7..).unwrap())
            .next()
            .unwrap();

        msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| e
                .author(|a| a
                    .icon_url("https://cdn.discordapp.com/avatars/286339479910875136/72a1e183852968a5b549d7c6391d2d4e.webp?size=128")
                    .name("Plurmp McFlurnten#7538")
                )
                .color(0x000000)
                .title(gallery.title)
                .url(format!("https://e-hentai.org/g/{}/{}/", gallery.gid, gallery.token))
                .description(format!("by {}", artist))
                .field(
                    "Japanese Title",
                    if let Some(jp) = gallery.title_jpn { jp } else { "None".to_string() },
                    true
                )
                .field(
                    "Tags",
                    gallery.tags.join(", "),
                    false
                )
                .image(gallery.thumb)
                .footer(|f| f.text(format!("{} pages | Rating: {} | Uploaded: ", gallery.filecount, gallery.rating)))
                .timestamp(
                    DateTime::<Utc>::from_utc(
                        NaiveDateTime::from_timestamp_opt(gallery.posted.parse().unwrap(), 0).unwrap(), Utc
                    )
                )
            )
            .reactions(vec![EmojiIdentifier::from_str("⬆️").unwrap(), EmojiIdentifier::from_str("⬇️").unwrap()])
        }).await?;
    } else {
        msg.channel_id.say(&ctx.http, "Multiple galleries found!").await?;
        msg.channel_id.say(
            &ctx.http,
            galleries
                .iter()
                .map(|g| format!("https://e-hentai.org/g/{}/{}/", g.gid, g.token))
                .collect::<Vec<_>>()
                .join("\n"),
        ).await?;
    }
    typing.stop();

    Ok(())
}

#[command]
pub async fn fullsearch(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    Ok(())
}

#[command]
pub async fn random(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}
