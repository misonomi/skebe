use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Work {
    pub path: String,
    pub nsfw: bool,
}
