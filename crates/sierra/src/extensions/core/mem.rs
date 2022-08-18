use super::{single_cell_identity, unpack_inputs};
use crate::define_extension_hierarchy;
use crate::extensions::{
    GenericExtension, InputError, NamedExtension, NoGenericArgsGenericExtension,
    NonBranchConcreteExtension, SpecializationError,
};
use crate::ids::ConcreteTypeId;
use crate::mem_cell::MemCell;
use crate::program::GenericArg;

define_extension_hierarchy! {
    pub enum MemExtension {
        StoreTemp(StoreTempGeneric),
        AlignTemps(AlignTempsGeneric),
        StoreLocal(StoreLocalGeneric),
        AllocLocals(AllocLocalsGeneric),
        Rename(RenameGeneric),
        Move(MoveGeneric)
    }, MemConcrete
}

/// Helper for extracting the type from the template arguments.
fn as_single_type(args: &[GenericArg]) -> Result<ConcreteTypeId, SpecializationError> {
    match args {
        [GenericArg::Type(ty)] => Ok(ty.clone()),
        _ => Err(SpecializationError::UnsupportedGenericArg),
    }
}

/// Extension for storing a deferred value into temporary memory.
#[derive(Default)]
pub struct StoreTempGeneric {}
impl NamedExtension for StoreTempGeneric {
    type Concrete = StoreTempConcrete;
    const NAME: &'static str = "store_temp";
    fn specialize(&self, args: &[GenericArg]) -> Result<Self::Concrete, SpecializationError> {
        Ok(StoreTempConcrete { _ty: as_single_type(args)? })
    }
}

pub struct StoreTempConcrete {
    _ty: ConcreteTypeId,
}
impl NonBranchConcreteExtension for StoreTempConcrete {
    fn non_branch_simulate(
        &self,
        inputs: Vec<Vec<MemCell>>,
    ) -> Result<Vec<Vec<MemCell>>, InputError> {
        single_cell_identity::<1>(inputs)
    }
}

/// Extension for aligning the temporary buffer for flow control merge.
#[derive(Default)]
pub struct AlignTempsGeneric {}
impl NamedExtension for AlignTempsGeneric {
    type Concrete = AlignTempsConcrete;
    const NAME: &'static str = "align_temps";
    fn specialize(&self, args: &[GenericArg]) -> Result<Self::Concrete, SpecializationError> {
        Ok(AlignTempsConcrete { _ty: as_single_type(args)? })
    }
}

pub struct AlignTempsConcrete {
    _ty: ConcreteTypeId,
}
impl NonBranchConcreteExtension for AlignTempsConcrete {
    fn non_branch_simulate(
        &self,
        inputs: Vec<Vec<MemCell>>,
    ) -> Result<Vec<Vec<MemCell>>, InputError> {
        unpack_inputs::<0>(inputs)?;
        Ok(vec![])
    }
}

/// Extension for storing a deferred value into local memory.
#[derive(Default)]
pub struct StoreLocalGeneric {}
impl NamedExtension for StoreLocalGeneric {
    type Concrete = StoreLocalConcrete;
    const NAME: &'static str = "store_local";
    fn specialize(&self, args: &[GenericArg]) -> Result<Self::Concrete, SpecializationError> {
        Ok(StoreLocalConcrete { _ty: as_single_type(args)? })
    }
}

pub struct StoreLocalConcrete {
    _ty: ConcreteTypeId,
}
impl NonBranchConcreteExtension for StoreLocalConcrete {
    fn non_branch_simulate(
        &self,
        inputs: Vec<Vec<MemCell>>,
    ) -> Result<Vec<Vec<MemCell>>, InputError> {
        single_cell_identity::<1>(inputs)
    }
}

/// Extension for allocating locals for later stores.
#[derive(Default)]
pub struct AllocLocalsGeneric {}
impl NoGenericArgsGenericExtension for AllocLocalsGeneric {
    type Concrete = AllocLocalsConcrete;
    const NAME: &'static str = "alloc_locals";
    fn specialize(&self) -> Self::Concrete {
        AllocLocalsConcrete {}
    }
}

pub struct AllocLocalsConcrete {}
impl NonBranchConcreteExtension for AllocLocalsConcrete {
    fn non_branch_simulate(
        &self,
        inputs: Vec<Vec<MemCell>>,
    ) -> Result<Vec<Vec<MemCell>>, InputError> {
        unpack_inputs::<0>(inputs)?;
        Ok(vec![])
    }
}

/// Extension for renaming an identifier - used to align identities for flow control merge.
#[derive(Default)]
pub struct RenameGeneric {}
impl NamedExtension for RenameGeneric {
    type Concrete = RenameConcrete;
    const NAME: &'static str = "rename";
    fn specialize(&self, args: &[GenericArg]) -> Result<Self::Concrete, SpecializationError> {
        Ok(RenameConcrete { _ty: as_single_type(args)? })
    }
}

pub struct RenameConcrete {
    _ty: ConcreteTypeId,
}
impl NonBranchConcreteExtension for RenameConcrete {
    fn non_branch_simulate(
        &self,
        inputs: Vec<Vec<MemCell>>,
    ) -> Result<Vec<Vec<MemCell>>, InputError> {
        single_cell_identity::<1>(inputs)
    }
}

/// Extension for making a type deferred for later store.
#[derive(Default)]
pub struct MoveGeneric {}
impl NamedExtension for MoveGeneric {
    type Concrete = MoveConcrete;
    const NAME: &'static str = "move";
    fn specialize(&self, args: &[GenericArg]) -> Result<Self::Concrete, SpecializationError> {
        Ok(MoveConcrete { _ty: as_single_type(args)? })
    }
}

pub struct MoveConcrete {
    _ty: ConcreteTypeId,
}
impl NonBranchConcreteExtension for MoveConcrete {
    fn non_branch_simulate(
        &self,
        inputs: Vec<Vec<MemCell>>,
    ) -> Result<Vec<Vec<MemCell>>, InputError> {
        single_cell_identity::<1>(inputs)
    }
}