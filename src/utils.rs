use skia_safe::{
    scalar,
    textlayout::{FontCollection, Paragraph, ParagraphBuilder, ParagraphStyle, TextStyle},
    FontMgr,
};

pub fn create_paragraph(text: &str, font_size: f32, line_height: Option<f32>) -> Paragraph {
    let mut font_collection = FontCollection::new();
    font_collection.set_default_font_manager(FontMgr::default(), "jetbrains mono");

    let mut style = ParagraphStyle::default();
    let mut text_style = TextStyle::default();
    text_style.set_font_size(font_size);

    if let Some(height) = line_height {
        text_style.set_height(height);
        text_style.set_height_override(true);
    }
    style.set_text_style(&text_style);

    let mut paragraph_builder = ParagraphBuilder::new(&style, font_collection);

    paragraph_builder.add_text(text);

    let mut paragraph = paragraph_builder.build();

    paragraph.layout(scalar::MAX);

    paragraph
}

pub fn get_cell_size(font_size: f32, line_height: Option<f32>) -> (f32, f32) {
    let paragraph = create_paragraph("T", font_size, line_height);
    (paragraph.min_intrinsic_width(), paragraph.height())
}
