use strum::{EnumIter, EnumString};

#[derive(Debug, PartialEq, Clone, EnumString, EnumIter)]
pub enum Difficulty {
    UNKNOWN,
    SIMPLE,
    EASY,
    MEDIUM,
    EXPERT,
}

