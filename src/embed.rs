use serde::Deserialize;
use serenity::{builder::CreateEmbed, utils::Colour};

#[derive(Deserialize, Clone)]
pub struct Embed {
    title: String,
    colour: Colour,
    sections: Vec<EmbedSection>,
}

#[derive(Deserialize, Clone)]
pub struct EmbedSection {
    title: String,
    content: String,
    inline: Option<bool>,
}

impl Embed {
    pub fn to_discord_embed(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::default();
        let fields = self.sections.iter().map(|section| section.to_field());

        embed
            .title(self.title.clone())
            .colour(self.colour)
            .fields(fields)
            .to_owned()
    }
}

impl EmbedSection {
    pub fn to_field(&self) -> (String, String, bool) {
        let inline = self.inline == Some(true);
        (self.title.clone(), self.content.clone(), inline)
    }
}
