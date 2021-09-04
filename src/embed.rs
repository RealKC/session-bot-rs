use serde::Deserialize;
use serenity::{builder::CreateEmbed, utils::Colour};

#[derive(Deserialize, Clone)]
pub struct Embed {
    title: String,
    colour: Colour,
    description: Option<String>,
    image: Option<String>,
    sections: Vec<Section>,
}

#[derive(Deserialize, Clone)]
struct Section {
    title: String,
    content: String,
}

impl Embed {
    pub fn to_discord_embed(&self) -> CreateEmbed {
        let mut embed = CreateEmbed::default();
        let fields = self.sections.iter().map(Section::to_field);

        embed
            .title(self.title.clone())
            .colour(self.colour)
            .fields(fields);

        if let Some(s) = &self.description {
            embed.description(s);
        }

        if let Some(s) = &self.image {
            embed.image(s);
        }

        embed
    }
}

impl Section {
    pub fn to_field(&self) -> (String, String, bool) {
        (self.title.clone(), self.content.clone(), false)
    }
}
