use std::marker::PhantomData;
use std::ptr;

use crate::error::SparseLinearAlgebraError;
// use crate::operators::BinaryOperatorType;
use crate::operators::{
    binary_operator::BinaryOperator, mask::VectorMask, options::OperatorOptions,
};
use crate::util::{ElementIndexSelector, ElementIndexSelectorGraphblasType, IndexConversion};
use crate::value_types::sparse_vector::SparseVector;
use crate::value_types::value_type::{AsBoolean, ValueType};

use crate::bindings_to_graphblas_implementation::{
    GrB_BinaryOp, GrB_Descriptor, GrB_Vector_assign_BOOL, GrB_Vector_assign_FP32,
    GrB_Vector_assign_FP64, GrB_Vector_assign_INT16, GrB_Vector_assign_INT32,
    GrB_Vector_assign_INT64, GrB_Vector_assign_INT8, GrB_Vector_assign_UINT16,
    GrB_Vector_assign_UINT32, GrB_Vector_assign_UINT64, GrB_Vector_assign_UINT8,
};

// TODO: explicitly define how dupicates are handled

// Implemented methods do not provide mutable access to GraphBLAS operators or options.
// Code review must consider that no mtable access is provided.
// https://doc.rust-lang.org/nomicon/send-and-sync.html
unsafe impl Send for InsertScalarIntoVector<bool, bool> {}
unsafe impl Send for InsertScalarIntoVector<u8, u8> {}
unsafe impl Send for InsertScalarIntoVector<u16, u16> {}
unsafe impl Send for InsertScalarIntoVector<u32, u32> {}
unsafe impl Send for InsertScalarIntoVector<u64, u64> {}
unsafe impl Send for InsertScalarIntoVector<i8, i8> {}
unsafe impl Send for InsertScalarIntoVector<i16, i16> {}
unsafe impl Send for InsertScalarIntoVector<i32, i32> {}
unsafe impl Send for InsertScalarIntoVector<i64, i64> {}
unsafe impl Send for InsertScalarIntoVector<f32, f32> {}
unsafe impl Send for InsertScalarIntoVector<f64, f64> {}

unsafe impl Sync for InsertScalarIntoVector<bool, bool> {}
unsafe impl Sync for InsertScalarIntoVector<u8, u8> {}
unsafe impl Sync for InsertScalarIntoVector<u16, u16> {}
unsafe impl Sync for InsertScalarIntoVector<u32, u32> {}
unsafe impl Sync for InsertScalarIntoVector<u64, u64> {}
unsafe impl Sync for InsertScalarIntoVector<i8, i8> {}
unsafe impl Sync for InsertScalarIntoVector<i16, i16> {}
unsafe impl Sync for InsertScalarIntoVector<i32, i32> {}
unsafe impl Sync for InsertScalarIntoVector<i64, i64> {}
unsafe impl Sync for InsertScalarIntoVector<f32, f32> {}
unsafe impl Sync for InsertScalarIntoVector<f64, f64> {}

#[derive(Debug, Clone)]
pub struct InsertScalarIntoVector<VectorToInsertInto: ValueType, ScalarToInsert: ValueType> {
    _vector_to_insert_into: PhantomData<VectorToInsertInto>,
    _scalar_to_insert: PhantomData<ScalarToInsert>,

    accumulator: GrB_BinaryOp, // optional accum for Z=accum(C,T), determines how results are written into the result matrix C
    options: GrB_Descriptor,
}

impl<VectorToInsertInto, ScalarToInsert> InsertScalarIntoVector<VectorToInsertInto, ScalarToInsert>
where
    VectorToInsertInto: ValueType,
    ScalarToInsert: ValueType,
{
    pub fn new(
        options: &OperatorOptions,
        accumulator: Option<
            &dyn BinaryOperator<ScalarToInsert, VectorToInsertInto, VectorToInsertInto>,
        >, // optional accum for Z=accum(C,T), determines how results are written into the result matrix C
    ) -> Self {
        let accumulator_to_use;
        match accumulator {
            Some(accumulator) => accumulator_to_use = accumulator.graphblas_type(),
            None => accumulator_to_use = ptr::null_mut(),
        }

        Self {
            accumulator: accumulator_to_use,
            options: options.to_graphblas_descriptor(),

            _vector_to_insert_into: PhantomData,
            _scalar_to_insert: PhantomData,
        }
    }
}

pub trait InsertScalarIntoVectorTrait<VectorToInsertInto, ScalarToInsert>
where
    VectorToInsertInto: ValueType,
    ScalarToInsert: ValueType,
{
    /// replace option applies to entire matrix_to_insert_to
    fn apply(
        &self,
        vector_to_insert_into: &mut SparseVector<VectorToInsertInto>,
        indices_to_insert_into: &ElementIndexSelector,
        scalar_to_insert: &ScalarToInsert,
    ) -> Result<(), SparseLinearAlgebraError>;

    /// mask and replace option apply to entire matrix_to_insert_to
    fn apply_with_mask<MaskValueType: ValueType, AsBool: AsBoolean<MaskValueType>>(
        &self,
        vector_to_insert_into: &mut SparseVector<VectorToInsertInto>,
        indices_to_insert_into: &ElementIndexSelector,
        scalar_to_insert: &ScalarToInsert,
        mask_for_vector_to_insert_into: &VectorMask<MaskValueType, AsBool>,
    ) -> Result<(), SparseLinearAlgebraError>;
}

macro_rules! implement_insert_scalar_into_vector_trait {
    (
        $value_type_vector_to_insert_into:ty, $value_type_scalar_to_insert:ty, $graphblas_insert_function:ident
    ) => {
        impl
            InsertScalarIntoVectorTrait<
                $value_type_vector_to_insert_into,
                $value_type_scalar_to_insert,
            >
            for InsertScalarIntoVector<
                $value_type_vector_to_insert_into,
                $value_type_scalar_to_insert,
            >
        {
            /// replace option applies to entire matrix_to_insert_to
            fn apply(
                &self,
                vector_to_insert_into: &mut SparseVector<$value_type_vector_to_insert_into>,
                indices_to_insert_into: &ElementIndexSelector,
                scalar_to_insert: &$value_type_scalar_to_insert,
            ) -> Result<(), SparseLinearAlgebraError> {
                let context = vector_to_insert_into.context();

                let number_of_indices_to_insert_into = indices_to_insert_into
                    .number_of_selected_elements(vector_to_insert_into.length()?)?
                    .to_graphblas_index()?;

                let indices_to_insert_into = indices_to_insert_into.to_graphblas_type()?;

                match indices_to_insert_into {
                    ElementIndexSelectorGraphblasType::Index(index) => {
                        context.call(|| unsafe {
                            $graphblas_insert_function(
                                vector_to_insert_into.graphblas_vector(),
                                ptr::null_mut(),
                                self.accumulator,
                                *scalar_to_insert,
                                index.as_ptr(),
                                number_of_indices_to_insert_into,
                                self.options,
                            )
                        })?;
                    }

                    ElementIndexSelectorGraphblasType::All(index) => {
                        context.call(|| unsafe {
                            $graphblas_insert_function(
                                vector_to_insert_into.graphblas_vector(),
                                ptr::null_mut(),
                                self.accumulator,
                                *scalar_to_insert,
                                index,
                                number_of_indices_to_insert_into,
                                self.options,
                            )
                        })?;
                    }
                }

                Ok(())
            }

            /// mask and replace option apply to entire matrix_to_insert_to
            fn apply_with_mask<MaskValueType: ValueType, AsBool: AsBoolean<MaskValueType>>(
                &self,
                vector_to_insert_into: &mut SparseVector<$value_type_vector_to_insert_into>,
                indices_to_insert_into: &ElementIndexSelector,
                scalar_to_insert: &$value_type_scalar_to_insert,
                mask_for_vector_to_insert_into: &VectorMask<MaskValueType, AsBool>,
            ) -> Result<(), SparseLinearAlgebraError> {
                let context = vector_to_insert_into.context();

                let number_of_indices_to_insert_into = indices_to_insert_into
                    .number_of_selected_elements(vector_to_insert_into.length()?)?
                    .to_graphblas_index()?;

                let indices_to_insert_into = indices_to_insert_into.to_graphblas_type()?;

                match indices_to_insert_into {
                    ElementIndexSelectorGraphblasType::Index(index) => {
                        context.call(|| unsafe {
                            $graphblas_insert_function(
                                vector_to_insert_into.graphblas_vector(),
                                mask_for_vector_to_insert_into.graphblas_vector(),
                                self.accumulator,
                                *scalar_to_insert,
                                index.as_ptr(),
                                number_of_indices_to_insert_into,
                                self.options,
                            )
                        })?;
                    }

                    ElementIndexSelectorGraphblasType::All(index) => {
                        context.call(|| unsafe {
                            $graphblas_insert_function(
                                vector_to_insert_into.graphblas_vector(),
                                mask_for_vector_to_insert_into.graphblas_vector(),
                                self.accumulator,
                                *scalar_to_insert,
                                index,
                                number_of_indices_to_insert_into,
                                self.options,
                            )
                        })?;
                    }
                }

                Ok(())
            }
        }
    };
}

implement_insert_scalar_into_vector_trait!(bool, bool, GrB_Vector_assign_BOOL);
implement_insert_scalar_into_vector_trait!(u8, u8, GrB_Vector_assign_UINT8);
implement_insert_scalar_into_vector_trait!(u16, u16, GrB_Vector_assign_UINT16);
implement_insert_scalar_into_vector_trait!(u32, u32, GrB_Vector_assign_UINT32);
implement_insert_scalar_into_vector_trait!(u64, u64, GrB_Vector_assign_UINT64);
implement_insert_scalar_into_vector_trait!(i8, i8, GrB_Vector_assign_INT8);
implement_insert_scalar_into_vector_trait!(i16, i16, GrB_Vector_assign_INT16);
implement_insert_scalar_into_vector_trait!(i32, i32, GrB_Vector_assign_INT32);
implement_insert_scalar_into_vector_trait!(i64, i64, GrB_Vector_assign_INT64);
implement_insert_scalar_into_vector_trait!(f32, f32, GrB_Vector_assign_FP32);
implement_insert_scalar_into_vector_trait!(f64, f64, GrB_Vector_assign_FP64);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::context::{Context, Mode};
    use crate::operators::binary_operator::First;
    use crate::util::ElementIndex;
    use crate::value_types::sparse_vector::{
        FromVectorElementList, GetVectorElementValue, VectorElementList,
    };

    #[test]
    fn test_insert_scalar_into_vector() {
        let context = Context::init_ready(Mode::NonBlocking).unwrap();

        let element_list = VectorElementList::<u8>::from_element_vector(vec![
            (1, 1).into(),
            (2, 2).into(),
            (4, 10).into(),
            (5, 11).into(),
        ]);

        let vector_length: usize = 10;
        let mut vector = SparseVector::<u8>::from_element_list(
            &context,
            &vector_length,
            &element_list,
            &First::<u8, u8, u8>::new(),
        )
        .unwrap();

        let mask_element_list = VectorElementList::<bool>::from_element_vector(vec![
            // (1, 1, true).into(),
            (2, true).into(),
            (4, true).into(),
            (5, true).into(),
        ]);
        let mask = SparseVector::<bool>::from_element_list(
            &context,
            &vector_length,
            &mask_element_list,
            &First::<bool, bool, bool>::new(),
        )
        .unwrap();

        let indices_to_insert: Vec<ElementIndex> = (0..3).collect();
        let indices_to_insert = ElementIndexSelector::Index(&indices_to_insert);

        let insert_operator = InsertScalarIntoVector::new(&OperatorOptions::new_default(), None);

        let scalar_to_insert: u8 = 8;

        insert_operator
            .apply(&mut vector, &indices_to_insert, &scalar_to_insert)
            .unwrap();

        assert_eq!(vector.number_of_stored_elements().unwrap(), 5);
        assert_eq!(vector.get_element_value(&0).unwrap(), 8);
        assert_eq!(vector.get_element_value(&2).unwrap(), 8);
        assert_eq!(vector.get_element_value(&3).unwrap(), 0);
        assert_eq!(vector.get_element_value(&5).unwrap(), 11);

        let mut vector = SparseVector::<u8>::from_element_list(
            &context,
            &vector_length,
            &element_list,
            &First::<u8, u8, u8>::new(),
        )
        .unwrap();

        insert_operator
            .apply_with_mask(
                &mut vector,
                &indices_to_insert,
                &scalar_to_insert,
                &mask.into(),
            )
            .unwrap();

        // println!("{}", vector);

        assert_eq!(vector.number_of_stored_elements().unwrap(), 4);
        assert_eq!(vector.get_element_value(&0).unwrap(), 0);
        assert_eq!(vector.get_element_value(&2).unwrap(), 8);
        assert_eq!(vector.get_element_value(&4).unwrap(), 10);
        assert_eq!(vector.get_element_value(&5).unwrap(), 11);
        assert_eq!(vector.get_element_value(&1).unwrap(), 1);
    }
}
