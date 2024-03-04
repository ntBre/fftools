use openff_toolkit::typing::engines::smirnoff::parameters::ParameterHandler;
use rdkit_rs::ROMol;

pub struct ParameterMap(pub(crate) Vec<(String, ROMol)>);

impl ParameterMap {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl From<ParameterHandler> for ParameterMap {
    fn from(ph: ParameterHandler) -> Self {
        Self(
            ph.parameters()
                .into_iter()
                .map(|p| (p.id(), ROMol::from_smarts(&p.smirks())))
                .collect(),
        )
    }
}
