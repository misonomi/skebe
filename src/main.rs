use egg_mode::{
    auth::{verify_tokens, Token},
    tweet::DraftTweet,
    KeyPair,
};
use tokio_compat_02::FutureExt;

use std::env;

mod skeb_ds;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let token = Token::Access {
        consumer: KeyPair::new(
            env::var("SKEBE_TWITTER_API_KEY").unwrap(),
            env::var("SKEBE_TWITTER_API_SECRET").unwrap(),
        ),
        access: KeyPair::new(
            env::var("SKEBE_TWITTER_ACCESS_TOKEN").unwrap(),
            env::var("SKEBE_TWITTER_ACCESS_SECRET").unwrap(),
        ),
    };

    let user = verify_tokens(&token).compat().await?;
    let stopper = user
        .status
        .as_ref()
        .map(|s| {
            s.entities
                .urls
                .get(0)
                .map(|u| u.expanded_url.as_ref().or(Some(&u.display_url)))
        })
        .flatten()
        .flatten();

    let works = reqwest::get(&format!(
        "https://skeb.jp/api/works?sort=date&genre=art&offset=0&limit={}",
        env::var("SKEBE_LIMIT").unwrap_or_else(|_| "50".to_string())
    ))
    .await?
    .json::<Vec<skeb_ds::Work>>()
    .await?
    .into_iter()
    .filter(|w| w.nsfw)
    .take_while(|w| match stopper {
        Some(s) => !s.ends_with(&w.path),
        None => false,
    })
    .collect::<Vec<skeb_ds::Work>>()
    .into_iter()
    .rev()
    .collect::<Vec<skeb_ds::Work>>();

    for w in works {
        DraftTweet::new(format!("https://skeb.jp{}", w.path))
            .send(&token)
            .compat()
            .await
            .expect("failed to tweet");
    }
    Ok(())
}
