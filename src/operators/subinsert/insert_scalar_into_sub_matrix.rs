use std::ptr;

use std::marker::PhantomData;

use crate::error::SparseLinearAlgebraError;
use crate::operators::{
    binary_operator::BinaryOperator, mask::MatrixMask, options::OperatorOptions,
};
use crate::value_types::sparse_matrix::SparseMatrix;

use crate::util::{ElementIndexSelector, ElementIndexSelectorGraphblasType, IndexConversion};
use crate::value_types::value_type::{AsBoolean, ValueType};

use crate::bindings_to_graphblas_implementation::{
    GrB_BinaryOp, GrB_Descriptor, GxB_Matrix_subassign_BOOL, GxB_Matrix_subassign_FP32,
    GxB_Matrix_subassign_FP64, GxB_Matrix_subassign_INT16, GxB_Matrix_subassign_INT32,
    GxB_Matrix_subassign_INT64, GxB_Matrix_subassign_INT8, GxB_Matrix_subassign_UINT16,
    GxB_Matrix_subassign_UINT32, GxB_Matrix_subassign_UINT64, GxB_Matrix_subassign_UINT8,
};

// TODO: explicitly define how dupicates are handled

// Implemented methods do not provide mutable access to GraphBLAS operators or options.
// Code review must consider that no mtable access is provided.
// https://doc.rust-lang.org/nomicon/send-and-sync.html
unsafe impl Send for InsertScalarIntoSubMatrix<bool, bool> {}
unsafe impl Send for InsertScalarIntoSubMatrix<u8, u8> {}
unsafe impl Send for InsertScalarIntoSubMatrix<u16, u16> {}
unsafe impl Send for InsertScalarIntoSubMatrix<u32, u32> {}
unsafe impl Send for InsertScalarIntoSubMatrix<u64, u64> {}
unsafe impl Send for InsertScalarIntoSubMatrix<i8, i8> {}
unsafe impl Send for InsertScalarIntoSubMatrix<i16, i16> {}
unsafe impl Send for InsertScalarIntoSubMatrix<i32, i32> {}
unsafe impl Send for InsertScalarIntoSubMatrix<i64, i64> {}
unsafe impl Send for InsertScalarIntoSubMatrix<f32, f32> {}
unsafe impl Send for InsertScalarIntoSubMatrix<f64, f64> {}

unsafe impl Sync for InsertScalarIntoSubMatrix<bool, bool> {}
unsafe impl Sync for InsertScalarIntoSubMatrix<u8, u8> {}
unsafe impl Sync for InsertScalarIntoSubMatrix<u16, u16> {}
unsafe impl Sync for InsertScalarIntoSubMatrix<u32, u32> {}
unsafe impl Sync for InsertScalarIntoSubMatrix<u64, u64> {}
unsafe impl Sync for InsertScalarIntoSubMatrix<i8, i8> {}
unsafe impl Sync for InsertScalarIntoSubMatrix<i16, i16> {}
unsafe impl Sync for InsertScalarIntoSubMatrix<i32, i32> {}
unsafe impl Sync for InsertScalarIntoSubMatrix<i64, i64> {}
unsafe impl Sync for InsertScalarIntoSubMatrix<f32, f32> {}
unsafe impl Sync for InsertScalarIntoSubMatrix<f64, f64> {}

#[derive(Debug, Clone)]
pub struct InsertScalarIntoSubMatrix<MatrixToInsertInto: ValueType, ScalarToInsert: ValueType> {
    _matrix_to_insert_into: PhantomData<MatrixToInsertInto>,
    _scalar_to_insert: PhantomData<ScalarToInsert>,

    accumulator: GrB_BinaryOp, // optional accum for Z=accum(C,T), determines how results are written into the result matrix C
    options: GrB_Descriptor,
}

impl<MatrixToInsertInto, ScalarToInsert>
    InsertScalarIntoSubMatrix<MatrixToInsertInto, ScalarToInsert>
where
    MatrixToInsertInto: ValueType,
    ScalarToInsert: ValueType,
{
    pub fn new(
        options: &OperatorOptions,
        accumulator: Option<
            &dyn BinaryOperator<ScalarToInsert, MatrixToInsertInto, MatrixToInsertInto>,
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

            _matrix_to_insert_into: PhantomData,
            _scalar_to_insert: PhantomData,
        }
    }
}

pub trait InsertScalarIntoSubMatrixTrait<MatrixToInsertInto, ScalarToInsert>
where
    MatrixToInsertInto: ValueType,
    ScalarToInsert: ValueType,
{
    /// replace option applies to entire matrix_to_insert_to
    fn apply(
        &self,
        matrix_to_insert_into: &mut SparseMatrix<MatrixToInsertInto>,
        rows_to_insert_into: &ElementIndexSelector, // length must equal row_height of matrix_to_insert
        columns_to_insert_into: &ElementIndexSelector, // length must equal column_width of matrix_to_insert
        scalar_to_insert: &ScalarToInsert,
    ) -> Result<(), SparseLinearAlgebraError>;

    /// mask and replace option apply to entire matrix_to_insert_to
    fn apply_with_mask<MaskValueType: ValueType, AsBool: AsBoolean<MaskValueType>>(
        &self,
        matrix_to_insert_into: &mut SparseMatrix<MatrixToInsertInto>,
        rows_to_insert_into: &ElementIndexSelector, // length must equal row_height of matrix_to_insert
        columns_to_insert_into: &ElementIndexSelector, // length must equal column_width of matrix_to_insert
        scalar_to_insert: &ScalarToInsert,
        mask_for_matrix_to_insert_into: &MatrixMask<MaskValueType, AsBool>,
    ) -> Result<(), SparseLinearAlgebraError>;
}

macro_rules! implement_insert_scalar_into_sub_matrix_trait {
    (
        $value_type_matrix_to_insert_into:ty, $value_type_scalar_to_insert:ty, $graphblas_insert_function:ident
    ) => {
        impl
            InsertScalarIntoSubMatrixTrait<
                $value_type_matrix_to_insert_into,
                $value_type_scalar_to_insert,
            >
            for InsertScalarIntoSubMatrix<
                $value_type_matrix_to_insert_into,
                $value_type_scalar_to_insert,
            >
        {
            /// replace option applies to entire matrix_to_insert_to
            fn apply(
                &self,
                matrix_to_insert_into: &mut SparseMatrix<$value_type_matrix_to_insert_into>,
                rows_to_insert_into: &ElementIndexSelector, // length must equal row_height of matrix_to_insert
                columns_to_insert_into: &ElementIndexSelector, // length must equal column_width of matrix_to_insert
                scalar_to_insert: &$value_type_scalar_to_insert,
            ) -> Result<(), SparseLinearAlgebraError> {
                let context = matrix_to_insert_into.context();

                let number_of_rows_to_insert_into = rows_to_insert_into
                    .number_of_selected_elements(matrix_to_insert_into.row_height()?)?
                    .to_graphblas_index()?;

                let number_of_columns_to_insert_into = columns_to_insert_into
                    .number_of_selected_elements(matrix_to_insert_into.column_width()?)?
                    .to_graphblas_index()?;

                let rows_to_insert_into = rows_to_insert_into.to_graphblas_type()?;
                let columns_to_insert_into = columns_to_insert_into.to_graphblas_type()?;

                match (rows_to_insert_into, columns_to_insert_into) {
                    (
                        ElementIndexSelectorGraphblasType::Index(row),
                        ElementIndexSelectorGraphblasType::Index(column),
                    ) => {
                        context.call(|| unsafe {
                            $graphblas_insert_function(
                                matrix_to_insert_into.graphblas_matrix(),
                                ptr::null_mut(),
                                self.accumulator,
                                *scalar_to_insert,
                                row.as_ptr(),
                                number_of_rows_to_insert_into,
                                column.as_ptr(),
                                number_of_columns_to_insert_into,
                                self.options,
                            )
                        })?;
                    }
                    (
                        ElementIndexSelectorGraphblasType::All(row),
                        ElementIndexSelectorGraphblasType::Index(column),
                    ) => {
                        context.call(|| unsafe {
                            $graphblas_insert_function(
                                matrix_to_insert_into.graphblas_matrix(),
                                ptr::null_mut(),
                                self.accumulator,
                                *scalar_to_insert,
                                row,
                                number_of_rows_to_insert_into,
                                column.as_ptr(),
                                number_of_columns_to_insert_into,
                                self.options,
                            )
                        })?;
                    }
                    (
                        ElementIndexSelectorGraphblasType::Index(row),
                        ElementIndexSelectorGraphblasType::All(column),
                    ) => {
                        context.call(|| unsafe {
                            $graphblas_insert_function(
                                matrix_to_insert_into.graphblas_matrix(),
                                ptr::null_mut(),
                                self.accumulator,
                                *scalar_to_insert,
                                row.as_ptr(),
                                number_of_rows_to_insert_into,
                                column,
                                number_of_columns_to_insert_into,
                                self.options,
                            )
                        })?;
                    }
                    (
                        ElementIndexSelectorGraphblasType::All(row),
                        ElementIndexSelectorGraphblasType::All(column),
                    ) => {
                        context.call(|| unsafe {
                            $graphblas_insert_function(
                                matrix_to_insert_into.graphblas_matrix(),
                                ptr::null_mut(),
                                self.accumulator,
                                *scalar_to_insert,
                                row,
                                number_of_rows_to_insert_into,
                                column,
                                number_of_columns_to_insert_into,
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
                matrix_to_insert_into: &mut SparseMatrix<$value_type_matrix_to_insert_into>,
                rows_to_insert_into: &ElementIndexSelector, // length must equal row_height of matrix_to_insert
                columns_to_insert_into: &ElementIndexSelector, // length must equal column_width of matrix_to_insert
                scalar_to_insert: &$value_type_scalar_to_insert,
                mask_for_matrix_to_insert_into: &MatrixMask<MaskValueType, AsBool>,
            ) -> Result<(), SparseLinearAlgebraError> {
                let context = matrix_to_insert_into.context();

                let number_of_rows_to_insert_into = rows_to_insert_into
                    .number_of_selected_elements(matrix_to_insert_into.row_height()?)?
                    .to_graphblas_index()?;

                let number_of_columns_to_insert_into = columns_to_insert_into
                    .number_of_selected_elements(matrix_to_insert_into.column_width()?)?
                    .to_graphblas_index()?;

                let rows_to_insert_into = rows_to_insert_into.to_graphblas_type()?;
                let columns_to_insert_into = columns_to_insert_into.to_graphblas_type()?;

                match (rows_to_insert_into, columns_to_insert_into) {
                    (
                        ElementIndexSelectorGraphblasType::Index(row),
                        ElementIndexSelectorGraphblasType::Index(column),
                    ) => {
                        context.call(|| unsafe {
                            $graphblas_insert_function(
                                matrix_to_insert_into.graphblas_matrix(),
                                mask_for_matrix_to_insert_into.graphblas_matrix(),
                                self.accumulator,
                                *scalar_to_insert,
                                row.as_ptr(),
                                number_of_rows_to_insert_into,
                                column.as_ptr(),
                                number_of_columns_to_insert_into,
                                self.options,
                            )
                        })?;
                    }
                    (
                        ElementIndexSelectorGraphblasType::All(row),
                        ElementIndexSelectorGraphblasType::Index(column),
                    ) => {
                        context.call(|| unsafe {
                            $graphblas_insert_function(
                                matrix_to_insert_into.graphblas_matrix(),
                                mask_for_matrix_to_insert_into.graphblas_matrix(),
                                self.accumulator,
                                *scalar_to_insert,
                                row,
                                number_of_rows_to_insert_into,
                                column.as_ptr(),
                                number_of_columns_to_insert_into,
                                self.options,
                            )
                        })?;
                    }
                    (
                        ElementIndexSelectorGraphblasType::Index(row),
                        ElementIndexSelectorGraphblasType::All(column),
                    ) => {
                        context.call(|| unsafe {
                            $graphblas_insert_function(
                                matrix_to_insert_into.graphblas_matrix(),
                                mask_for_matrix_to_insert_into.graphblas_matrix(),
                                self.accumulator,
                                *scalar_to_insert,
                                row.as_ptr(),
                                number_of_rows_to_insert_into,
                                column,
                                number_of_columns_to_insert_into,
                                self.options,
                            )
                        })?;
                    }
                    (
                        ElementIndexSelectorGraphblasType::All(row),
                        ElementIndexSelectorGraphblasType::All(column),
                    ) => {
                        context.call(|| unsafe {
                            $graphblas_insert_function(
                                matrix_to_insert_into.graphblas_matrix(),
                                mask_for_matrix_to_insert_into.graphblas_matrix(),
                                self.accumulator,
                                *scalar_to_insert,
                                row,
                                number_of_rows_to_insert_into,
                                column,
                                number_of_columns_to_insert_into,
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

implement_insert_scalar_into_sub_matrix_trait!(bool, bool, GxB_Matrix_subassign_BOOL);
implement_insert_scalar_into_sub_matrix_trait!(u8, u8, GxB_Matrix_subassign_UINT8);
implement_insert_scalar_into_sub_matrix_trait!(u16, u16, GxB_Matrix_subassign_UINT16);
implement_insert_scalar_into_sub_matrix_trait!(u32, u32, GxB_Matrix_subassign_UINT32);
implement_insert_scalar_into_sub_matrix_trait!(u64, u64, GxB_Matrix_subassign_UINT64);
implement_insert_scalar_into_sub_matrix_trait!(i8, i8, GxB_Matrix_subassign_INT8);
implement_insert_scalar_into_sub_matrix_trait!(i16, i16, GxB_Matrix_subassign_INT16);
implement_insert_scalar_into_sub_matrix_trait!(i32, i32, GxB_Matrix_subassign_INT32);
implement_insert_scalar_into_sub_matrix_trait!(i64, i64, GxB_Matrix_subassign_INT64);
implement_insert_scalar_into_sub_matrix_trait!(f32, f32, GxB_Matrix_subassign_FP32);
implement_insert_scalar_into_sub_matrix_trait!(f64, f64, GxB_Matrix_subassign_FP64);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::context::{Context, Mode};
    use crate::operators::binary_operator::First;

    use crate::util::ElementIndex;
    use crate::value_types::sparse_matrix::{
        FromMatrixElementList, GetMatrixElementValue, MatrixElementList, Size,
    };

    #[test]
    fn test_insert_scalar_into_matrix() {
        let context = Context::init_ready(Mode::NonBlocking).unwrap();

        let element_list = MatrixElementList::<u8>::from_element_vector(vec![
            (1, 1, 1).into(),
            (2, 2, 2).into(),
            (2, 4, 10).into(),
            // (2, 5, 11).into(),
        ]);

        let matrix_size: Size = (10, 15).into();
        let mut matrix = SparseMatrix::<u8>::from_element_list(
            &context,
            &matrix_size,
            &element_list,
            &First::<u8, u8, u8>::new(),
        )
        .unwrap();

        let mask_element_list = MatrixElementList::<bool>::from_element_vector(vec![
            // (1, 1, true).into(),
            (2, 2, true).into(),
            (2, 4, true).into(),
            (2, 5, true).into(),
        ]);
        let mask = SparseMatrix::<bool>::from_element_list(
            &context,
            &(3, 6).into(),
            &mask_element_list,
            &First::<bool, bool, bool>::new(),
        )
        .unwrap();

        let rows_to_insert: Vec<ElementIndex> = (0..3).collect();
        let rows_to_insert = ElementIndexSelector::Index(&rows_to_insert);
        let columns_to_insert: Vec<ElementIndex> = (0..6).collect();
        let columns_to_insert = ElementIndexSelector::Index(&columns_to_insert);

        let insert_operator = InsertScalarIntoSubMatrix::new(&OperatorOptions::new_default(), None);

        let scalar_to_insert: u8 = 8;

        insert_operator
            .apply(
                &mut matrix,
                &rows_to_insert,
                &columns_to_insert,
                &scalar_to_insert,
            )
            .unwrap();

        assert_eq!(matrix.number_of_stored_elements().unwrap(), 18);
        assert_eq!(matrix.get_element_value(&(0, 0).into()).unwrap(), 8);
        assert_eq!(matrix.get_element_value(&(2, 2).into()).unwrap(), 8);
        assert_eq!(matrix.get_element_value(&(2, 4).into()).unwrap(), 8);
        assert_eq!(matrix.get_element_value(&(9, 14).into()).unwrap(), 0);

        let mut matrix = SparseMatrix::<u8>::from_element_list(
            &context,
            &matrix_size,
            &element_list,
            &First::<u8, u8, u8>::new(),
        )
        .unwrap();

        insert_operator
            .apply_with_mask(
                &mut matrix,
                &rows_to_insert,
                &columns_to_insert,
                &scalar_to_insert,
                &mask.into(),
            )
            .unwrap();

        println!("{}", matrix);

        assert_eq!(matrix.number_of_stored_elements().unwrap(), 4);
        assert_eq!(matrix.get_element_value(&(0, 0).into()).unwrap(), 0);
        assert_eq!(matrix.get_element_value(&(2, 2).into()).unwrap(), 8);
        assert_eq!(matrix.get_element_value(&(2, 4).into()).unwrap(), 8);
        assert_eq!(matrix.get_element_value(&(2, 5).into()).unwrap(), 8);
        assert_eq!(matrix.get_element_value(&(1, 1).into()).unwrap(), 1);
    }
}
