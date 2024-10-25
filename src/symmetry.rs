#[derive(Debug, Clone, PartialEq)]
pub enum Symmetry {
    NONE,
    ROTATE90,
    ROTATE180,
    MIRROR,
    FLIP,
    RANDOM,
}