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

pub struct Dimensions {
    pub width: f32,
    pub height: f32,
}

impl Dimensions {
    pub fn from((width, height): (f32, f32)) -> Dimensions {
        Dimensions { width, height }
    }
}

pub struct CropLine {
    pub orientation: CropLineOrientation,
    pub offset: f32,
    pub length: f32,
}

pub enum CropLineOrientation {
    Horizontal,
    Vertical,
}

pub struct CardPlacement {
    pub x: f32,
    pub y: f32,
    pub rotate: Option<f32>,
    pub reflect: Option<Axis>,
}

pub enum Axis {
    Horizontal,
    Vertical,
}
