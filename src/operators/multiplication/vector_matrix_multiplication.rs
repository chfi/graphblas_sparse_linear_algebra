use std::ptr;

use std::marker::PhantomData;

use crate::error::SparseLinearAlgebraError;
use crate::operators::binary_operator::BinaryOperator;
use crate::operators::semiring::Semiring;
use crate::operators::{mask::VectorMask, options::OperatorOptions};
use crate::value_types::sparse_matrix::SparseMatrix;
use crate::value_types::sparse_vector::SparseVector;
use crate::value_types::value_type::{AsBoolean, ValueType};

use crate::bindings_to_graphblas_implementation::{
    GrB_BinaryOp, GrB_Descriptor, GrB_Semiring, GrB_vxm,
};

// Implemented methods do not provide mutable access to GraphBLAS operators or options.
// Code review must consider that no mtable access is provided.
// https://doc.rust-lang.org/nomicon/send-and-sync.html
unsafe impl Send for VectorMatrixMultiplicationOperator<bool, bool, bool> {}
unsafe impl Send for VectorMatrixMultiplicationOperator<u8, u8, u8> {}
unsafe impl Send for VectorMatrixMultiplicationOperator<u16, u16, u16> {}
unsafe impl Send for VectorMatrixMultiplicationOperator<u32, u32, u32> {}
unsafe impl Send for VectorMatrixMultiplicationOperator<u64, u64, u64> {}
unsafe impl Send for VectorMatrixMultiplicationOperator<i8, i8, i8> {}
unsafe impl Send for VectorMatrixMultiplicationOperator<i16, i16, i16> {}
unsafe impl Send for VectorMatrixMultiplicationOperator<i32, i32, i32> {}
unsafe impl Send for VectorMatrixMultiplicationOperator<i64, i64, i64> {}
unsafe impl Send for VectorMatrixMultiplicationOperator<f32, f32, f32> {}
unsafe impl Send for VectorMatrixMultiplicationOperator<f64, f64, f64> {}

unsafe impl Sync for VectorMatrixMultiplicationOperator<bool, bool, bool> {}
unsafe impl Sync for VectorMatrixMultiplicationOperator<u8, u8, u8> {}
unsafe impl Sync for VectorMatrixMultiplicationOperator<u16, u16, u16> {}
unsafe impl Sync for VectorMatrixMultiplicationOperator<u32, u32, u32> {}
unsafe impl Sync for VectorMatrixMultiplicationOperator<u64, u64, u64> {}
unsafe impl Sync for VectorMatrixMultiplicationOperator<i8, i8, i8> {}
unsafe impl Sync for VectorMatrixMultiplicationOperator<i16, i16, i16> {}
unsafe impl Sync for VectorMatrixMultiplicationOperator<i32, i32, i32> {}
unsafe impl Sync for VectorMatrixMultiplicationOperator<i64, i64, i64> {}
unsafe impl Sync for VectorMatrixMultiplicationOperator<f32, f32, f32> {}
unsafe impl Sync for VectorMatrixMultiplicationOperator<f64, f64, f64> {}

// TODO: review the use of &'a dyn Trait, removing dynamic dispatch could provide a performance gain. (it might be negated if cloning is necessary though)
// https://www.joshmcguigan.com/blog/cost-of-indirection-rust/
#[derive(Debug, Clone)]
pub struct VectorMatrixMultiplicationOperator<Multiplier, Multiplicant, Product>
where
    Multiplier: ValueType,
    Multiplicant: ValueType,
    Product: ValueType,
{
    _multiplier: PhantomData<Multiplier>,
    _multiplicant: PhantomData<Multiplicant>,
    _product: PhantomData<Product>,

    accumulator: GrB_BinaryOp, // optional accum for Z=accum(C,T), determines how results are written into the result matrix C
    semiring: GrB_Semiring, // defines '+' and '*' for A*B (not optional for GrB_mxm)
    options: GrB_Descriptor,
}

impl<Multiplier, Multiplicant, Product>
    VectorMatrixMultiplicationOperator<Multiplier, Multiplicant, Product>
where
    Multiplier: ValueType,
    Multiplicant: ValueType,
    Product: ValueType,
{
    pub fn new(
        // semiring: Box<dyn Semiring<Multiplier, Multiplicant, Product>>,
        // defines '+' and '*' for A*B (not optional for GrB_mxm)
        semiring: &dyn Semiring<Multiplier, Multiplicant, Product>,
        options: &OperatorOptions,
        accumulator: Option<Box<dyn BinaryOperator<Product, Product, Product>>>, // optional accum for Z=accum(C,T), determines how results are written into the result matrix C
    ) -> Self {
        let accumulator_to_use;
        match accumulator {
            Some(accumulator) => {
                accumulator_to_use = accumulator.graphblas_type()
            }
            None => accumulator_to_use = ptr::null_mut(),
        }

        Self {
            accumulator: accumulator_to_use,
            semiring: semiring.graphblas_type(),
            options: options.to_graphblas_descriptor(),

            _multiplier: PhantomData,
            _multiplicant: PhantomData,
            _product: PhantomData,
        }
    }

    // TODO: consider a version where the resulting product matrix is generated in the function body
    pub fn apply(
        &self,
        multiplier: &SparseVector<Multiplier>,
        multiplicant: &SparseMatrix<Multiplicant>,
        product: &mut SparseVector<Product>,
    ) -> Result<(), SparseLinearAlgebraError> {
        let context = product.context();

        context.call(|| unsafe {
            GrB_vxm(
                product.graphblas_vector(),
                ptr::null_mut(),
                self.accumulator,
                self.semiring,
                multiplier.graphblas_vector(),
                multiplicant.graphblas_matrix(),
                self.options,
            )
        })?;

        Ok(())
    }

    pub fn apply_with_mask<
        MaskValueType: ValueType,
        AsBool: AsBoolean<MaskValueType>,
    >(
        &self,
        mask: &VectorMask<MaskValueType, AsBool>,
        multiplier: &SparseVector<Multiplier>,
        multiplicant: &SparseMatrix<Multiplicant>,
        product: &mut SparseVector<Product>,
    ) -> Result<(), SparseLinearAlgebraError> {
        let context = product.context();

        context.call(|| unsafe {
            GrB_vxm(
                product.graphblas_vector(),
                mask.graphblas_vector(),
                self.accumulator,
                self.semiring,
                multiplier.graphblas_vector(),
                multiplicant.graphblas_matrix(),
                self.options,
            )
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::context::{Context, Mode};
    use crate::operators::binary_operator::First;
    use crate::operators::binary_operator::Plus;
    use crate::operators::semiring::PlusTimes;
    use crate::value_types::sparse_matrix::{
        FromMatrixElementList, MatrixElementList, Size,
    };
    use crate::value_types::sparse_vector::{
        FromVectorElementList, GetVectorElementList, GetVectorElementValue,
        VectorElementList,
    };

    #[test]
    fn test_multiplication_with_plus_times() {
        let context = Context::init_ready(Mode::NonBlocking).unwrap();

        let semiring = Box::new(PlusTimes::<f32, f32, f32>::new());
        let options = OperatorOptions::new_default();
        let matrix_multiplier =
            VectorMatrixMultiplicationOperator::<f32, f32, f32>::new(
                semiring.clone(),
                options.clone(),
                None,
            );

        let length = 2;
        let size: Size = (length, length).into();

        let multiplier = SparseVector::<f32>::new(&context, &length).unwrap();
        let multiplicant = SparseMatrix::<f32>::new(&context, &size).unwrap();
        let mut product = multiplier.clone();

        // Test multiplication of empty matrices
        matrix_multiplier
            .apply(&multiplier, &multiplicant, &mut product)
            .unwrap();
        let element_list = product.get_element_list().unwrap();

        assert_eq!(product.number_of_stored_elements().unwrap(), 0);
        assert_eq!(element_list.length(), 0);
        assert_eq!(product.get_element_value(&1).unwrap(), 0.); // NoValue

        let multiplier_element_list =
            VectorElementList::<f32>::from_element_vector(vec![
                (0, 1.0).into(),
                (1, 2.0).into(),
            ]);
        let multiplier = SparseVector::<f32>::from_element_list(
            &context,
            &length,
            &multiplier_element_list,
            &First::<f32, f32, f32>::new(),
        )
        .unwrap();

        let multiplicant_element_list =
            MatrixElementList::<f32>::from_element_vector(vec![
                (0, 0, 5.0).into(),
                (1, 0, 6.0).into(),
                (0, 1, 7.0).into(),
                (1, 1, 8.0).into(),
            ]);
        let multiplicant = SparseMatrix::<f32>::from_element_list(
            &context,
            &size,
            &multiplicant_element_list,
            &First::<f32, f32, f32>::new(),
        )
        .unwrap();

        // Test multiplication of full matrices
        matrix_multiplier
            .apply(&multiplier, &multiplicant, &mut product)
            .unwrap();

        assert_eq!(product.get_element_value(&0).unwrap(), 17.);
        assert_eq!(product.get_element_value(&1).unwrap(), 23.);

        // TODO: this test is not generic over column/row storage format.
        // Equality checks should be done at a matrix level, since the ordering of the element list is not guaranteed.
        let expected_product =
            VectorElementList::<f32>::from_element_vector(vec![
                (0, 17.).into(),
                (1, 23.).into(),
            ]);
        let product_element_list = product.get_element_list().unwrap();
        assert_eq!(expected_product, product_element_list);

        // test the use of an accumulator
        let accumulator = Box::new(Plus::<f32, f32, f32>::new());
        let matrix_multiplier_with_accumulator =
            VectorMatrixMultiplicationOperator::<f32, f32, f32>::new(
                semiring.clone(),
                options.clone(),
                Some(accumulator),
            );

        matrix_multiplier_with_accumulator
            .apply(&multiplier, &multiplicant, &mut product)
            .unwrap();

        assert_eq!(product.get_element_value(&0).unwrap(), 17. * 2.);
        assert_eq!(product.get_element_value(&1).unwrap(), 23. * 2.);

        // test the use of a mask
        let mask_element_list =
            VectorElementList::<u8>::from_element_vector(vec![
                (0, 3).into(),
                (1, 0).into(),
            ]);
        let mask = SparseVector::<u8>::from_element_list(
            &context,
            &length,
            &mask_element_list,
            &First::<u8, u8, u8>::new(),
        )
        .unwrap();

        let matrix_multiplier =
            VectorMatrixMultiplicationOperator::<f32, f32, f32>::new(
                semiring.clone(),
                options.clone(),
                None,
            );

        let mut product = SparseVector::<f32>::new(&context, &length).unwrap();

        matrix_multiplier
            .apply_with_mask(
                &mask.into(),
                &multiplier,
                &multiplicant,
                &mut product,
            )
            .unwrap();

        assert_eq!(product.get_element_value(&0).unwrap(), 17.);
        assert_eq!(product.get_element_value(&1).unwrap(), 0.);
    }
}
