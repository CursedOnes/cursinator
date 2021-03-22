use crate::addon::AddonID;
use crate::addon::files::AddonFile;


pub fn assume_deps(deps: impl Iterator<Item=AddonID>, install_ops: &mut Vec<(AddonID,AddonFile)>) {
    // version picking for deps:
    // 1. if explicit version for parent, filter to-install dep versions before specific date
    //for dep in 
}
