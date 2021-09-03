use serde::Deserialize;
use serenity::{builder::CreateEmbed, utils::Colour};

#[derive(Deserialize, Clone)]
pub struct Embed {
    title: String,
    colour: Colour,
    description: Option<String>,
    image: Option<String>,
    sections: Vec<EmbedSection>,
}

#[derive(Deserialize, Clone)]
pub struct EmbedSection {
    title: String,
    content: String,
}

impl Embed {
    pub fn to_discord_embed(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::default();
        let fields = self.sections.iter().map(EmbedSection::to_field);

        embed = embed
            .title(self.title.clone())
            .colour(self.colour)
            .fields(fields)
            .to_owned();

        embed = match &self.description {
            None => embed,
            Some(s) => embed.description(s).to_owned(),
        };

        embed = match &self.image {
            None => embed,
            Some(s) => embed.image(s).to_owned(),
        };

        embed
    }
}

impl EmbedSection {
    pub fn to_field(&self) -> (String, String, bool) {
        (self.title.clone(), self.content.clone(), false)
    }
}
