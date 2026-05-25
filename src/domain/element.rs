#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Element {
    H,
    C,
    N,
    O,
    Na,
    Mg,
    Al,
    Si,
    S,
    Cl,
    K,
    Ca,
    Fe,
    Ni,
}

#[derive(Debug, Clone)]
pub struct ElementFraction {
    pub element: Element,
    pub mass_fraction: f64,
}