use crate::{Font, FontProvider, FontStretch, FontStyle};
use fontdb::{Database, ID};
use rustybuzz::ttf_parser;
use svgtypes::FontFamily;

impl FontProvider for Database {
    fn find_font(&self, font: &Font) -> Option<ID> {
        let mut name_list = Vec::new();
        for family in &font.families {
            name_list.push(match family {
                FontFamily::Serif => fontdb::Family::Serif,
                FontFamily::SansSerif => fontdb::Family::SansSerif,
                FontFamily::Cursive => fontdb::Family::Cursive,
                FontFamily::Fantasy => fontdb::Family::Fantasy,
                FontFamily::Monospace => fontdb::Family::Monospace,
                FontFamily::Named(s) => fontdb::Family::Name(s),
            });
        }

        // Use the default font as fallback.
        name_list.push(fontdb::Family::Serif);

        let stretch = match font.stretch {
            FontStretch::UltraCondensed => fontdb::Stretch::UltraCondensed,
            FontStretch::ExtraCondensed => fontdb::Stretch::ExtraCondensed,
            FontStretch::Condensed => fontdb::Stretch::Condensed,
            FontStretch::SemiCondensed => fontdb::Stretch::SemiCondensed,
            FontStretch::Normal => fontdb::Stretch::Normal,
            FontStretch::SemiExpanded => fontdb::Stretch::SemiExpanded,
            FontStretch::Expanded => fontdb::Stretch::Expanded,
            FontStretch::ExtraExpanded => fontdb::Stretch::ExtraExpanded,
            FontStretch::UltraExpanded => fontdb::Stretch::UltraExpanded,
        };

        let style = match font.style {
            FontStyle::Normal => fontdb::Style::Normal,
            FontStyle::Italic => fontdb::Style::Italic,
            FontStyle::Oblique => fontdb::Style::Oblique,
        };

        let query = fontdb::Query {
            families: &name_list,
            weight: fontdb::Weight(font.weight),
            stretch,
            style,
        };

        let id = self.query(&query);
        if id.is_none() {
            log::warn!(
                "No match for '{}' font-family.",
                font.families
                    .iter()
                    .map(|f| f.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        id
    }

    fn find_fallback_font(&self, c: char, base_font_id: ID, used_fonts: &[ID]) -> Option<ID> {
        // Iterate over fonts and check if any of them support the specified char.
        for face in self.faces() {
            // Ignore fonts, that were used for shaping already.
            if used_fonts.contains(&face.id) {
                continue;
            }

            // Check that the new face has the same style.
            let base_face = self.face(base_font_id)?;
            if base_face.style != face.style
                && base_face.weight != face.weight
                && base_face.stretch != face.stretch
            {
                continue;
            }

            let has_char = self
                .with_face_data(face.id, |font_data, face_index| -> Option<bool> {
                    let font = ttf_parser::Face::parse(font_data, face_index).ok()?;
                    font.glyph_index(c)?;
                    Some(true)
                })
                .flatten()
                .unwrap_or(false);

            if !has_char {
                continue;
            }

            let base_family = base_face
                .families
                .iter()
                .find(|f| f.1 == fontdb::Language::English_UnitedStates)
                .unwrap_or(&base_face.families[0]);

            let new_family = face
                .families
                .iter()
                .find(|f| f.1 == fontdb::Language::English_UnitedStates)
                .unwrap_or(&base_face.families[0]);

            log::warn!("Fallback from {} to {}.", base_family.0, new_family.0);
            return Some(face.id);
        }

        None
    }

    fn with_database(&self, f: &mut dyn FnMut(&Database)) {
        f(self);
    }
}
