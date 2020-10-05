#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Reaction(pub Type, pub Level);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Type {
    #[serde(rename = "34S(a,p)37Cl")]
    S34,
    #[serde(rename = "34Cl(a,p)37Ar")]
    Cl34,
    #[serde(rename = "34Ar(a,p)37K")]
    Ar34,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Level {
    Overall,
    ExcitedState(u64),
}
