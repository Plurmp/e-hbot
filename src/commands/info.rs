use serenity::framework::standard::macros::command;
use serenity::framework::standard::{Args, CommandResult};
use serenity::model::prelude::*;
use serenity::prelude::*;

#[command]
pub async fn help(ctx: &Context, msg: &Message, mut args: Args) -> CommandResult {
    Ok(())
}

#[command]
pub async fn botinfo(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
pub async fn badtags(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}

#[command]
pub async fn warningtags(ctx: &Context, msg: &Message) -> CommandResult {
    Ok(())
}
