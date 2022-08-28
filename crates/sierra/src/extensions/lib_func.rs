use std::collections::HashMap;

use super::error::{ExtensionError, SpecializationError};
use crate::ids::{ConcreteTypeId, FunctionId, GenericLibFuncId, GenericTypeId};
use crate::program::{Function, GenericArg};

pub type FunctionMap = HashMap<FunctionId, Function>;
/// Mapping from the arguments for generating a concrete type (the generic-id and the arguments) to
/// the concrete-id that points to it.
pub type ConcreteTypeIdMap<'a> = HashMap<(GenericTypeId, &'a [GenericArg]), ConcreteTypeId>;
/// Context required for specialization process.
pub struct SpecializationContext<'a> {
    pub functions: &'a FunctionMap,
    pub concrete_type_ids: &'a ConcreteTypeIdMap<'a>,
}
impl SpecializationContext<'_> {
    /// Returns concrete type id or an error if missing.
    pub fn get_concrete_type(
        &self,
        id: GenericTypeId,
        args: &[GenericArg],
    ) -> Result<ConcreteTypeId, SpecializationError> {
        self.concrete_type_ids
            .get(&(id.clone(), args))
            .ok_or_else(|| SpecializationError::TypeWasNotDeclared(id, args.to_vec()))
            .cloned()
    }
    /// Returns the concrete id of a generic-type-id wrapping the type of a concrete-type-id.
    pub fn get_wrapped_concrete_type(
        &self,
        id: GenericTypeId,
        wrapped: ConcreteTypeId,
    ) -> Result<ConcreteTypeId, SpecializationError> {
        self.get_concrete_type(id, &[GenericArg::Type(wrapped)])
    }
}

/// Trait for implementing a libfunc specialization generator.
pub trait GenericLibFunc: Sized {
    type Concrete: ConcreteLibFunc;

    /// Instantiates the libfunc by id.
    fn by_id(id: &GenericLibFuncId) -> Option<Self>;
    /// Creates the specialization with the template arguments.
    fn specialize(
        &self,
        context: SpecializationContext<'_>,
        args: &[GenericArg],
    ) -> Result<Self::Concrete, SpecializationError>;
}

/// Trait for introducing helper methods on GenericLibFunc.
pub trait GenericLibFuncEx: GenericLibFunc {
    fn specialize_by_id(
        context: SpecializationContext<'_>,
        libfunc_id: &GenericLibFuncId,
        args: &[GenericArg],
    ) -> Result<Self::Concrete, ExtensionError>;
}
impl<TGenericLibFunc: GenericLibFunc> GenericLibFuncEx for TGenericLibFunc {
    fn specialize_by_id(
        context: SpecializationContext<'_>,
        libfunc_id: &GenericLibFuncId,
        args: &[GenericArg],
    ) -> Result<TGenericLibFunc::Concrete, ExtensionError> {
        Self::by_id(libfunc_id)
            .ok_or_else(move || ExtensionError::LibFuncSpecialization {
                libfunc_id: libfunc_id.clone(),
                error: SpecializationError::UnsupportedId,
            })?
            .specialize(context, args)
            .map_err(move |error| ExtensionError::LibFuncSpecialization {
                libfunc_id: libfunc_id.clone(),
                error,
            })
    }
}

/// Trait for implementing a specialization generator with with a simple id.
pub trait NamedLibFunc: Default {
    type Concrete: ConcreteLibFunc;
    const ID: GenericLibFuncId;
    /// Creates the specialization with the template arguments.
    fn specialize(
        &self,
        context: SpecializationContext<'_>,
        args: &[GenericArg],
    ) -> Result<Self::Concrete, SpecializationError>;
}
impl<TNamedLibFunc: NamedLibFunc> GenericLibFunc for TNamedLibFunc {
    type Concrete = <Self as NamedLibFunc>::Concrete;

    fn by_id(id: &GenericLibFuncId) -> Option<Self> {
        if &Self::ID == id { Some(Self::default()) } else { None }
    }

    fn specialize(
        &self,
        context: SpecializationContext<'_>,
        args: &[GenericArg],
    ) -> Result<Self::Concrete, SpecializationError> {
        <Self as NamedLibFunc>::specialize(self, context, args)
    }
}

/// Trait for implementing a specialization generator with no generic arguments.
pub trait NoGenericArgsGenericLibFunc: Default {
    type Concrete: ConcreteLibFunc;
    const ID: GenericLibFuncId;
    fn specialize(
        &self,
        context: SpecializationContext<'_>,
    ) -> Result<Self::Concrete, SpecializationError>;
}
impl<T: NoGenericArgsGenericLibFunc> NamedLibFunc for T {
    type Concrete = <Self as NoGenericArgsGenericLibFunc>::Concrete;
    const ID: GenericLibFuncId = <Self as NoGenericArgsGenericLibFunc>::ID;

    fn specialize(
        &self,
        context: SpecializationContext<'_>,
        args: &[GenericArg],
    ) -> Result<Self::Concrete, SpecializationError> {
        if args.is_empty() {
            self.specialize(context)
        } else {
            Err(SpecializationError::WrongNumberOfGenericArgs)
        }
    }
}

/// Trait for a specialized library function.
pub trait ConcreteLibFunc {
    /// The input types for calling the library function.
    fn input_types(&self) -> Vec<ConcreteTypeId>;
    /// The output types returning from library function per branch.
    fn output_types(&self) -> Vec<Vec<ConcreteTypeId>>;
    /// The index of the fallthrough branch of the library function if any.
    fn fallthrough(&self) -> Option<usize>;
}

/// Trait for a non branch specialized libfunc.
pub trait NonBranchConcreteLibFunc {
    fn input_types(&self) -> Vec<ConcreteTypeId>;
    fn output_types(&self) -> Vec<ConcreteTypeId>;
}
impl<TNonBranchConcreteLibFunc: NonBranchConcreteLibFunc> ConcreteLibFunc
    for TNonBranchConcreteLibFunc
{
    fn input_types(&self) -> Vec<ConcreteTypeId> {
        <Self as NonBranchConcreteLibFunc>::input_types(self)
    }
    fn output_types(&self) -> Vec<Vec<ConcreteTypeId>> {
        vec![<Self as NonBranchConcreteLibFunc>::output_types(self)]
    }
    fn fallthrough(&self) -> Option<usize> {
        Some(0)
    }
}

/// Forms a concrete library function type from an enum of library calls.
/// The new enum implements ConcreteLibFunc.
/// All the variant types must also implement ConcreteLibFunc.
/// Usage example:
/// ```ignore
/// define_concrete_libfunc_hierarchy! {
///     pub enum MyLibFunc {
///       LF0(LibFunc0),
///       LF1(LibFunc1),
///     }
/// }
/// ```
#[macro_export]
macro_rules! define_concrete_libfunc_hierarchy {
    (pub enum $name:ident { $($variant_name:ident ($variant:ty),)* }) => {
        #[allow(clippy::enum_variant_names)]
        pub enum $name {
            $($variant_name ($variant),)*
        }

        impl $crate::extensions::ConcreteLibFunc for $name {
            fn input_types(&self) -> Vec<$crate::ids::ConcreteTypeId> {
                match self {
                    $(Self::$variant_name(value) =>
                        <$variant as $crate::extensions::ConcreteLibFunc>::input_types(value)),*
                }
            }
            fn output_types(&self) -> Vec<Vec<$crate::ids::ConcreteTypeId>> {
                match self {
                    $(Self::$variant_name(value) =>
                        <$variant as $crate::extensions::ConcreteLibFunc>::output_types(value)),*
                }
            }
            fn fallthrough(&self) -> Option<usize> {
                match self {
                    $(Self::$variant_name(value) =>
                        <$variant as $crate::extensions::ConcreteLibFunc>::fallthrough(value)),*
                }
            }
        }
    }
}

/// Forms a libfunc type from an enum of libfuncs.
/// The new enum implements GenericLibFunc.
/// All the variant types must also implement GenericLibFunc.
/// Usage example:
/// ```ignore
/// define_libfunc_hierarchy! {
///     pub enum MyLibFunc {
///       LF0(LibFunc0),
///       LF1(LibFunc1),
///     }, MyLibFuncConcrete
/// }
/// ```
#[macro_export]
macro_rules! define_libfunc_hierarchy {
    (pub enum $name:ident { $($variant_name:ident ($variant:ty),)* },
    $concrete_name:ident) => {
        #[allow(clippy::enum_variant_names)]
        pub enum $name {
            $($variant_name ($variant)),*
        }

        impl $crate::extensions::GenericLibFunc for $name {
            type Concrete = $concrete_name;
            fn by_id(id: &$crate::ids::GenericLibFuncId) -> Option<Self> {
                $(
                    if let Some(res) = <$variant>::by_id(id){
                        return Some(Self::$variant_name(res));
                    }
                )*
                None
            }
            fn specialize(
                    &self,
                    context: $crate::extensions::lib_func::SpecializationContext<'_>,
                    args: &[$crate::program::GenericArg],
            ) -> Result<Self::Concrete, $crate::extensions::SpecializationError>{
                match self {
                    $(
                        Self::$variant_name(value) => {
                            let inner = <$variant as GenericLibFunc>::specialize(value, context, args)?;
                            Ok(Self::Concrete::$variant_name(inner.into()))
                        }
                    ),*
                }
            }
        }

        $crate::define_concrete_libfunc_hierarchy! {
            pub enum $concrete_name {
                $($variant_name (<$variant as GenericLibFunc> ::Concrete),)*
            }
        }
    }
}