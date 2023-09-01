#[derive(Clone)]
pub struct Sheet {
    pub page_size: Dimensions,
    pub card_size: Dimensions,
    pub crop_lines: Vec<CropLine>,
    pub cards: Vec<CardPlacement>,
}

impl Sheet {
    pub fn num_cards(&self) -> usize {
        self.cards.len()
    }
}

#[derive(Clone)]
pub struct Dimensions {
    pub width: f32,
    pub height: f32,
}

impl Dimensions {
    pub fn from((width, height): (f32, f32)) -> Dimensions {
        Dimensions { width, height }
    }
}

#[derive(Clone)]
pub struct CropLine {
    pub orientation: CropLineOrientation,
    pub offset: f32,
    pub length: f32,
}

#[derive(Clone)]
pub enum CropLineOrientation {
    Horizontal,
    Vertical,
}

#[derive(Clone)]
pub struct CardPlacement {
    pub x: f32,
    pub y: f32,
    pub rotate: Option<f32>,
    pub reflect: Option<Axis>,
}

#[derive(Clone)]
pub enum Axis {
    Horizontal,
    Vertical,
}
