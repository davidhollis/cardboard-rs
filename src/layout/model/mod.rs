use self::{geometry::Geometry, elements::Element};

pub mod elements;
pub mod geometry;
pub mod styles;

#[derive(knuffel::Decode, PartialEq, Eq, Debug)]
pub struct Layout {
    #[knuffel(child)]
    pub geometry: Geometry,
    #[knuffel(children)]
    pub elements: Vec<Element>,
}

#[cfg(test)]
mod tests {
    use crate::layout::{model::{geometry::{Insets, Geometry}, elements::{shapes::{Background, Rectangle}, Element, text::Text, Frame, containers::Box}, styles::{solid::Solid, PathStyle, only_if::{OnlyIf, OnlyIfOperator}, stroke::{Stroke, DashPattern}, TextStyle, font::Font, color::{ColorRef, Color}}}, templates::TemplateAwareString};

    use super::Layout;

    const EXAMPLE_DOCUMENT: &str = r#"
    geometry {
        width 825
        height 1125
        cut 37
        safe 75
    }
    background {
        solid "white"
    }
    rectangle x=1 y=2 w=3 h=4
    rectangle x=5 y=6 w=7 h=8 {
        only-if "some text"
        stroke 3 "black"
        solid "rgba(110, 120, 130, 255)"
    }
    text "some text" {
        frame x=100 y=200 w=300 h=400
        only-if "some {{other}} text" "in" "xxx" "yyy" "zzz"
    }
    box x=50 y=50 w=100 h=100 {
        rectangle x=1 y=2 w=3 h=4 {
            stroke 2 "black" {
                pattern "---  .. "
            }
        }
        text "some text" {
            frame x=10 y=20 w=30 h=40
            font "Fira Code"
        }
    }
    "#;

    #[test]
    fn it_loads_a_layout_file() -> miette::Result<()> {
        let layout: Layout = knuffel::parse("example.kdl", EXAMPLE_DOCUMENT)?;

        assert_eq!(
            layout,
            Layout {
                geometry: Geometry {
                    width: 825,
                    height: 1125,
                    cut: Insets::uniform(37),
                    safe: Insets::uniform(75)
                },
                elements: vec![
                    Element::Background(Background {
                        style: vec![
                            PathStyle::Solid(Solid {
                                color: ColorRef::Named(TemplateAwareString::new("white".to_string())),
                            }),
                        ],
                    }),
                    Element::Rectangle(Rectangle {
                        x: 1,
                        y: 2,
                        w: 3,
                        h: 4,
                        style: vec![],
                    }),
                    Element::Rectangle(Rectangle {
                        x: 5,
                        y: 6,
                        w: 7,
                        h: 8,
                        style: vec![
                            PathStyle::OnlyIf(OnlyIf {
                                left: TemplateAwareString::new("some text".to_string()),
                                op: None,
                                right: vec![],
                            }),
                            PathStyle::Stroke(Stroke {
                                width: 3,
                                color: ColorRef::Named(TemplateAwareString::new("black".to_string())),
                                pattern: DashPattern::Solid,
                            }),
                            PathStyle::Solid(Solid {
                                color: ColorRef::Static(Color::RGBA(110, 120, 130, 255)),
                            }),
                        ],
                    }),
                    Element::Text(Text {
                        contents: TemplateAwareString::new("some text".to_string()),
                        frame: Frame {
                            x: 100,
                            y: 200,
                            w: 300,
                            h: 400,
                        },
                        style: vec![
                            TextStyle::OnlyIf(OnlyIf {
                                left: TemplateAwareString::new("some {{other}} text".to_string()),
                                op: Some(OnlyIfOperator::In),
                                right: vec![
                                    TemplateAwareString::new("xxx".to_string()),
                                    TemplateAwareString::new("yyy".to_string()),
                                    TemplateAwareString::new("zzz".to_string()),
                                ],
                            }),
                        ],
                    }),
                    Element::Box(Box {
                        x: 50,
                        y: 50,
                        w: 100,
                        h: 100,
                        contents: vec![
                            Element::Rectangle(Rectangle {
                                x: 1,
                                y: 2,
                                w: 3,
                                h: 4,
                                style: vec![
                                    PathStyle::Stroke(
                                        Stroke {
                                            width: 2,
                                            color: ColorRef::Named(TemplateAwareString::new("black".to_string())),
                                            pattern: DashPattern::Dashed(vec![9, 2, 2, 1]),
                                        }
                                    )
                                ],
                            }),
                            Element::Text(Text {
                                contents: TemplateAwareString::new("some text".to_string()),
                                frame: Frame {
                                    x: 10,
                                    y: 20,
                                    w: 30,
                                    h: 40,
                                },
                                style: vec![
                                    TextStyle::Font(Font {
                                        name: "Fira Code".to_string(),
                                    }),
                                ],
                            }),
                        ],
                    })
                ],
            }
        );

        Ok(())
    }
}
