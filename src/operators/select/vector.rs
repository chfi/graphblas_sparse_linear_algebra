use std::ptr;

use std::marker::PhantomData;

use crate::error::SparseLinearAlgebraError;
use crate::operators::{
    binary_operator::BinaryOperator, mask::VectorMask, options::OperatorOptions,
};

use crate::value_types::sparse_scalar::{SetScalarValue, SparseScalar};
use crate::value_types::sparse_vector::SparseVector;

use crate::value_types::value_type::{AsBoolean, ValueType};

use crate::bindings_to_graphblas_implementation::{
    GrB_BinaryOp, GrB_Descriptor, GxB_EQ_THUNK, GxB_EQ_ZERO, GxB_GE_THUNK, GxB_GE_ZERO,
    GxB_GT_THUNK, GxB_GT_ZERO, GxB_LE_THUNK, GxB_LE_ZERO, GxB_LT_THUNK, GxB_LT_ZERO, GxB_NE_THUNK,
    GxB_NONZERO, GxB_Vector_select,
};

// Implemented methods do not provide mutable access to GraphBLAS operators or options.
// Code review must consider that no mtable access is provided.
// https://doc.rust-lang.org/nomicon/send-and-sync.html
unsafe impl Send for VectorSelector<bool> {}
unsafe impl Send for VectorSelector<u8> {}
unsafe impl Send for VectorSelector<u16> {}
unsafe impl Send for VectorSelector<u32> {}
unsafe impl Send for VectorSelector<u64> {}
unsafe impl Send for VectorSelector<i8> {}
unsafe impl Send for VectorSelector<i16> {}
unsafe impl Send for VectorSelector<i32> {}
unsafe impl Send for VectorSelector<i64> {}
unsafe impl Send for VectorSelector<f32> {}
unsafe impl Send for VectorSelector<f64> {}

unsafe impl Sync for VectorSelector<bool> {}
unsafe impl Sync for VectorSelector<u8> {}
unsafe impl Sync for VectorSelector<u16> {}
unsafe impl Sync for VectorSelector<u32> {}
unsafe impl Sync for VectorSelector<u64> {}
unsafe impl Sync for VectorSelector<i8> {}
unsafe impl Sync for VectorSelector<i16> {}
unsafe impl Sync for VectorSelector<i32> {}
unsafe impl Sync for VectorSelector<i64> {}
unsafe impl Sync for VectorSelector<f32> {}
unsafe impl Sync for VectorSelector<f64> {}

#[derive(Debug, Clone)]
pub struct VectorSelector<T: ValueType> {
    _value: PhantomData<T>,

    accumulator: GrB_BinaryOp, // optional accum for Z=accum(C,T), determines how results are written into the result matrix C
    options: GrB_Descriptor,
}

impl<T: ValueType> VectorSelector<T> {
    pub fn new(
        options: &OperatorOptions,
        accumulator: Option<&dyn BinaryOperator<T, T, T>>, // optional accum for Z=accum(C,T), determines how results are written into the result matrix C
    ) -> Self {
        let accumulator_to_use;
        match accumulator {
            Some(accumulator) => accumulator_to_use = accumulator.graphblas_type(),
            None => accumulator_to_use = ptr::null_mut(),
        }

        Self {
            accumulator: accumulator_to_use,
            options: options.to_graphblas_descriptor(),

            _value: PhantomData,
        }
    }
}

macro_rules! implement_scalar_selector {
    ($value_type:ty, $selector_trait:ident, $method_name:ident, $method_name_with_mask:ident, $graphblas_operator:ident) => {
        impl $selector_trait<$value_type> for VectorSelector<$value_type> {
            fn $method_name(
                &self,
                argument: &SparseVector<$value_type>,
                product: &mut SparseVector<$value_type>,
                scalar: &$value_type,
            ) -> Result<(), SparseLinearAlgebraError> {
                let context = product.context();
                let mut sparse_scalar = SparseScalar::<$value_type>::new(&context)?;
                sparse_scalar.set_value(scalar)?;

                context.call(|| unsafe {
                    GxB_Vector_select(
                        product.graphblas_vector(),
                        ptr::null_mut(),
                        self.accumulator,
                        $graphblas_operator,
                        argument.graphblas_vector(),
                        sparse_scalar.graphblas_scalar(),
                        self.options,
                    )
                })?;

                Ok(())
            }

            fn $method_name_with_mask<
                MaskValueType: ValueType,
                AsBool: AsBoolean<MaskValueType>,
            >(
                &self,
                argument: &SparseVector<$value_type>,
                product: &mut SparseVector<$value_type>,
                scalar: &$value_type,
                _mask: &VectorMask<MaskValueType, AsBool>,
            ) -> Result<(), SparseLinearAlgebraError> {
                let context = product.context();
                let mut sparse_scalar = SparseScalar::<$value_type>::new(&context)?;
                sparse_scalar.set_value(scalar)?;

                context.call(|| unsafe {
                    GxB_Vector_select(
                        product.graphblas_vector(),
                        ptr::null_mut(),
                        self.accumulator,
                        $graphblas_operator,
                        argument.graphblas_vector(),
                        sparse_scalar.graphblas_scalar(),
                        self.options,
                    )
                })?;

                Ok(())
            }
        }
    };
}

pub trait SelectVectorNotEqualToScalar<T: ValueType> {
    fn not_equal_to_scalar(
        &self,
        argument: &SparseVector<T>,
        product: &mut SparseVector<T>,
        scalar: &T,
    ) -> Result<(), SparseLinearAlgebraError>;

    fn not_equal_to_scalar_with_mask<MaskValueType: ValueType, AsBool: AsBoolean<MaskValueType>>(
        &self,
        argument: &SparseVector<T>,
        product: &mut SparseVector<T>,
        scalar: &T,
        mask: &VectorMask<MaskValueType, AsBool>,
    ) -> Result<(), SparseLinearAlgebraError>;
}

implement_scalar_selector!(
    bool,
    SelectVectorNotEqualToScalar,
    not_equal_to_scalar,
    not_equal_to_scalar_with_mask,
    GxB_NE_THUNK
);
implement_scalar_selector!(
    i8,
    SelectVectorNotEqualToScalar,
    not_equal_to_scalar,
    not_equal_to_scalar_with_mask,
    GxB_NE_THUNK
);
implement_scalar_selector!(
    i16,
    SelectVectorNotEqualToScalar,
    not_equal_to_scalar,
    not_equal_to_scalar_with_mask,
    GxB_NE_THUNK
);
implement_scalar_selector!(
    i32,
    SelectVectorNotEqualToScalar,
    not_equal_to_scalar,
    not_equal_to_scalar_with_mask,
    GxB_NE_THUNK
);
implement_scalar_selector!(
    i64,
    SelectVectorNotEqualToScalar,
    not_equal_to_scalar,
    not_equal_to_scalar_with_mask,
    GxB_NE_THUNK
);
implement_scalar_selector!(
    u8,
    SelectVectorNotEqualToScalar,
    not_equal_to_scalar,
    not_equal_to_scalar_with_mask,
    GxB_NE_THUNK
);
implement_scalar_selector!(
    u16,
    SelectVectorNotEqualToScalar,
    not_equal_to_scalar,
    not_equal_to_scalar_with_mask,
    GxB_NE_THUNK
);
implement_scalar_selector!(
    u32,
    SelectVectorNotEqualToScalar,
    not_equal_to_scalar,
    not_equal_to_scalar_with_mask,
    GxB_NE_THUNK
);
implement_scalar_selector!(
    u64,
    SelectVectorNotEqualToScalar,
    not_equal_to_scalar,
    not_equal_to_scalar_with_mask,
    GxB_NE_THUNK
);
implement_scalar_selector!(
    f32,
    SelectVectorNotEqualToScalar,
    not_equal_to_scalar,
    not_equal_to_scalar_with_mask,
    GxB_NE_THUNK
);
implement_scalar_selector!(
    f64,
    SelectVectorNotEqualToScalar,
    not_equal_to_scalar,
    not_equal_to_scalar_with_mask,
    GxB_NE_THUNK
);

pub trait SelectVectorEqualToScalar<T: ValueType> {
    fn equal_to_scalar(
        &self,
        argument: &SparseVector<T>,
        product: &mut SparseVector<T>,
        scalar: &T,
    ) -> Result<(), SparseLinearAlgebraError>;

    fn equal_to_scalar_with_mask<MaskValueType: ValueType, AsBool: AsBoolean<MaskValueType>>(
        &self,
        argument: &SparseVector<T>,
        product: &mut SparseVector<T>,
        scalar: &T,
        mask: &VectorMask<MaskValueType, AsBool>,
    ) -> Result<(), SparseLinearAlgebraError>;
}

implement_scalar_selector!(
    bool,
    SelectVectorEqualToScalar,
    equal_to_scalar,
    equal_to_scalar_with_mask,
    GxB_EQ_THUNK
);
implement_scalar_selector!(
    i8,
    SelectVectorEqualToScalar,
    equal_to_scalar,
    equal_to_scalar_with_mask,
    GxB_EQ_THUNK
);
implement_scalar_selector!(
    i16,
    SelectVectorEqualToScalar,
    equal_to_scalar,
    equal_to_scalar_with_mask,
    GxB_EQ_THUNK
);
implement_scalar_selector!(
    i32,
    SelectVectorEqualToScalar,
    equal_to_scalar,
    equal_to_scalar_with_mask,
    GxB_EQ_THUNK
);
implement_scalar_selector!(
    i64,
    SelectVectorEqualToScalar,
    equal_to_scalar,
    equal_to_scalar_with_mask,
    GxB_EQ_THUNK
);
implement_scalar_selector!(
    u8,
    SelectVectorEqualToScalar,
    equal_to_scalar,
    equal_to_scalar_with_mask,
    GxB_EQ_THUNK
);
implement_scalar_selector!(
    u16,
    SelectVectorEqualToScalar,
    equal_to_scalar,
    equal_to_scalar_with_mask,
    GxB_EQ_THUNK
);
implement_scalar_selector!(
    u32,
    SelectVectorEqualToScalar,
    equal_to_scalar,
    equal_to_scalar_with_mask,
    GxB_EQ_THUNK
);
implement_scalar_selector!(
    u64,
    SelectVectorEqualToScalar,
    equal_to_scalar,
    equal_to_scalar_with_mask,
    GxB_EQ_THUNK
);
implement_scalar_selector!(
    f32,
    SelectVectorEqualToScalar,
    equal_to_scalar,
    equal_to_scalar_with_mask,
    GxB_EQ_THUNK
);
implement_scalar_selector!(
    f64,
    SelectVectorEqualToScalar,
    equal_to_scalar,
    equal_to_scalar_with_mask,
    GxB_EQ_THUNK
);

pub trait SelectVectorGreaterThanScalar<T: ValueType> {
    fn greater_than_scalar(
        &self,
        argument: &SparseVector<T>,
        product: &mut SparseVector<T>,
        scalar: &T,
    ) -> Result<(), SparseLinearAlgebraError>;

    fn greater_than_scalar_with_mask<MaskValueType: ValueType, AsBool: AsBoolean<MaskValueType>>(
        &self,
        argument: &SparseVector<T>,
        product: &mut SparseVector<T>,
        scalar: &T,
        mask: &VectorMask<MaskValueType, AsBool>,
    ) -> Result<(), SparseLinearAlgebraError>;
}

implement_scalar_selector!(
    bool,
    SelectVectorGreaterThanScalar,
    greater_than_scalar,
    greater_than_scalar_with_mask,
    GxB_GT_THUNK
);
implement_scalar_selector!(
    i8,
    SelectVectorGreaterThanScalar,
    greater_than_scalar,
    greater_than_scalar_with_mask,
    GxB_GT_THUNK
);
implement_scalar_selector!(
    i16,
    SelectVectorGreaterThanScalar,
    greater_than_scalar,
    greater_than_scalar_with_mask,
    GxB_GT_THUNK
);
implement_scalar_selector!(
    i32,
    SelectVectorGreaterThanScalar,
    greater_than_scalar,
    greater_than_scalar_with_mask,
    GxB_GT_THUNK
);
implement_scalar_selector!(
    i64,
    SelectVectorGreaterThanScalar,
    greater_than_scalar,
    greater_than_scalar_with_mask,
    GxB_GT_THUNK
);
implement_scalar_selector!(
    u8,
    SelectVectorGreaterThanScalar,
    greater_than_scalar,
    greater_than_scalar_with_mask,
    GxB_GT_THUNK
);
implement_scalar_selector!(
    u16,
    SelectVectorGreaterThanScalar,
    greater_than_scalar,
    greater_than_scalar_with_mask,
    GxB_GT_THUNK
);
implement_scalar_selector!(
    u32,
    SelectVectorGreaterThanScalar,
    greater_than_scalar,
    greater_than_scalar_with_mask,
    GxB_GT_THUNK
);
implement_scalar_selector!(
    u64,
    SelectVectorGreaterThanScalar,
    greater_than_scalar,
    greater_than_scalar_with_mask,
    GxB_GT_THUNK
);
implement_scalar_selector!(
    f32,
    SelectVectorGreaterThanScalar,
    greater_than_scalar,
    greater_than_scalar_with_mask,
    GxB_GT_THUNK
);
implement_scalar_selector!(
    f64,
    SelectVectorGreaterThanScalar,
    greater_than_scalar,
    greater_than_scalar_with_mask,
    GxB_GT_THUNK
);

pub trait SelectVectorGreaterThanOrEqualToScalar<T: ValueType> {
    fn greater_than_or_equal_to_scalar(
        &self,
        argument: &SparseVector<T>,
        product: &mut SparseVector<T>,
        scalar: &T,
    ) -> Result<(), SparseLinearAlgebraError>;

    fn greater_than_or_equal_to_scalar_with_mask<
        MaskValueType: ValueType,
        AsBool: AsBoolean<MaskValueType>,
    >(
        &self,
        argument: &SparseVector<T>,
        product: &mut SparseVector<T>,
        scalar: &T,
        mask: &VectorMask<MaskValueType, AsBool>,
    ) -> Result<(), SparseLinearAlgebraError>;
}

implement_scalar_selector!(
    bool,
    SelectVectorGreaterThanOrEqualToScalar,
    greater_than_or_equal_to_scalar,
    greater_than_or_equal_to_scalar_with_mask,
    GxB_GE_THUNK
);
implement_scalar_selector!(
    i8,
    SelectVectorGreaterThanOrEqualToScalar,
    greater_than_or_equal_to_scalar,
    greater_than_or_equal_to_scalar_with_mask,
    GxB_GE_THUNK
);
implement_scalar_selector!(
    i16,
    SelectVectorGreaterThanOrEqualToScalar,
    greater_than_or_equal_to_scalar,
    greater_than_or_equal_to_scalar_with_mask,
    GxB_GE_THUNK
);
implement_scalar_selector!(
    i32,
    SelectVectorGreaterThanOrEqualToScalar,
    greater_than_or_equal_to_scalar,
    greater_than_or_equal_to_scalar_with_mask,
    GxB_GE_THUNK
);
implement_scalar_selector!(
    i64,
    SelectVectorGreaterThanOrEqualToScalar,
    greater_than_or_equal_to_scalar,
    greater_than_or_equal_to_scalar_with_mask,
    GxB_GE_THUNK
);
implement_scalar_selector!(
    u8,
    SelectVectorGreaterThanOrEqualToScalar,
    greater_than_or_equal_to_scalar,
    greater_than_or_equal_to_scalar_with_mask,
    GxB_GE_THUNK
);
implement_scalar_selector!(
    u16,
    SelectVectorGreaterThanOrEqualToScalar,
    greater_than_or_equal_to_scalar,
    greater_than_or_equal_to_scalar_with_mask,
    GxB_GE_THUNK
);
implement_scalar_selector!(
    u32,
    SelectVectorGreaterThanOrEqualToScalar,
    greater_than_or_equal_to_scalar,
    greater_than_or_equal_to_scalar_with_mask,
    GxB_GE_THUNK
);
implement_scalar_selector!(
    u64,
    SelectVectorGreaterThanOrEqualToScalar,
    greater_than_or_equal_to_scalar,
    greater_than_or_equal_to_scalar_with_mask,
    GxB_GE_THUNK
);
implement_scalar_selector!(
    f32,
    SelectVectorGreaterThanOrEqualToScalar,
    greater_than_or_equal_to_scalar,
    greater_than_or_equal_to_scalar_with_mask,
    GxB_GE_THUNK
);
implement_scalar_selector!(
    f64,
    SelectVectorGreaterThanOrEqualToScalar,
    greater_than_or_equal_to_scalar,
    greater_than_or_equal_to_scalar_with_mask,
    GxB_GE_THUNK
);

pub trait SelectVectorLessThanScalar<T: ValueType> {
    fn less_than_scalar(
        &self,
        argument: &SparseVector<T>,
        product: &mut SparseVector<T>,
        scalar: &T,
    ) -> Result<(), SparseLinearAlgebraError>;

    fn less_than_scalar_with_mask<MaskValueType: ValueType, AsBool: AsBoolean<MaskValueType>>(
        &self,
        argument: &SparseVector<T>,
        product: &mut SparseVector<T>,
        scalar: &T,
        mask: &VectorMask<MaskValueType, AsBool>,
    ) -> Result<(), SparseLinearAlgebraError>;
}

implement_scalar_selector!(
    bool,
    SelectVectorLessThanScalar,
    less_than_scalar,
    less_than_scalar_with_mask,
    GxB_LT_THUNK
);
implement_scalar_selector!(
    i8,
    SelectVectorLessThanScalar,
    less_than_scalar,
    less_than_scalar_with_mask,
    GxB_LT_THUNK
);
implement_scalar_selector!(
    i16,
    SelectVectorLessThanScalar,
    less_than_scalar,
    less_than_scalar_with_mask,
    GxB_LT_THUNK
);
implement_scalar_selector!(
    i32,
    SelectVectorLessThanScalar,
    less_than_scalar,
    less_than_scalar_with_mask,
    GxB_LT_THUNK
);
implement_scalar_selector!(
    i64,
    SelectVectorLessThanScalar,
    less_than_scalar,
    less_than_scalar_with_mask,
    GxB_LT_THUNK
);
implement_scalar_selector!(
    u8,
    SelectVectorLessThanScalar,
    less_than_scalar,
    less_than_scalar_with_mask,
    GxB_LT_THUNK
);
implement_scalar_selector!(
    u16,
    SelectVectorLessThanScalar,
    less_than_scalar,
    less_than_scalar_with_mask,
    GxB_LT_THUNK
);
implement_scalar_selector!(
    u32,
    SelectVectorLessThanScalar,
    less_than_scalar,
    less_than_scalar_with_mask,
    GxB_LT_THUNK
);
implement_scalar_selector!(
    u64,
    SelectVectorLessThanScalar,
    less_than_scalar,
    less_than_scalar_with_mask,
    GxB_LT_THUNK
);
implement_scalar_selector!(
    f32,
    SelectVectorLessThanScalar,
    less_than_scalar,
    less_than_scalar_with_mask,
    GxB_LT_THUNK
);
implement_scalar_selector!(
    f64,
    SelectVectorLessThanScalar,
    less_than_scalar,
    less_than_scalar_with_mask,
    GxB_LT_THUNK
);

pub trait SelectVectorLessThanOrEqualToScalar<T: ValueType> {
    fn less_than_or_equal_to_scalar(
        &self,
        argument: &SparseVector<T>,
        product: &mut SparseVector<T>,
        scalar: &T,
    ) -> Result<(), SparseLinearAlgebraError>;

    fn less_than_less_than_or_equal_to_scalar_with_mask<
        MaskValueType: ValueType,
        AsBool: AsBoolean<MaskValueType>,
    >(
        &self,
        argument: &SparseVector<T>,
        product: &mut SparseVector<T>,
        scalar: &T,
        mask: &VectorMask<MaskValueType, AsBool>,
    ) -> Result<(), SparseLinearAlgebraError>;
}

implement_scalar_selector!(
    bool,
    SelectVectorLessThanOrEqualToScalar,
    less_than_or_equal_to_scalar,
    less_than_less_than_or_equal_to_scalar_with_mask,
    GxB_LE_THUNK
);
implement_scalar_selector!(
    i8,
    SelectVectorLessThanOrEqualToScalar,
    less_than_or_equal_to_scalar,
    less_than_less_than_or_equal_to_scalar_with_mask,
    GxB_LE_THUNK
);
implement_scalar_selector!(
    i16,
    SelectVectorLessThanOrEqualToScalar,
    less_than_or_equal_to_scalar,
    less_than_less_than_or_equal_to_scalar_with_mask,
    GxB_LE_THUNK
);
implement_scalar_selector!(
    i32,
    SelectVectorLessThanOrEqualToScalar,
    less_than_or_equal_to_scalar,
    less_than_less_than_or_equal_to_scalar_with_mask,
    GxB_LE_THUNK
);
implement_scalar_selector!(
    i64,
    SelectVectorLessThanOrEqualToScalar,
    less_than_or_equal_to_scalar,
    less_than_less_than_or_equal_to_scalar_with_mask,
    GxB_LE_THUNK
);
implement_scalar_selector!(
    u8,
    SelectVectorLessThanOrEqualToScalar,
    less_than_or_equal_to_scalar,
    less_than_less_than_or_equal_to_scalar_with_mask,
    GxB_LE_THUNK
);
implement_scalar_selector!(
    u16,
    SelectVectorLessThanOrEqualToScalar,
    less_than_or_equal_to_scalar,
    less_than_less_than_or_equal_to_scalar_with_mask,
    GxB_LE_THUNK
);
implement_scalar_selector!(
    u32,
    SelectVectorLessThanOrEqualToScalar,
    less_than_or_equal_to_scalar,
    less_than_less_than_or_equal_to_scalar_with_mask,
    GxB_LE_THUNK
);
implement_scalar_selector!(
    u64,
    SelectVectorLessThanOrEqualToScalar,
    less_than_or_equal_to_scalar,
    less_than_less_than_or_equal_to_scalar_with_mask,
    GxB_LE_THUNK
);
implement_scalar_selector!(
    f32,
    SelectVectorLessThanOrEqualToScalar,
    less_than_or_equal_to_scalar,
    less_than_less_than_or_equal_to_scalar_with_mask,
    GxB_LE_THUNK
);
implement_scalar_selector!(
    f64,
    SelectVectorLessThanOrEqualToScalar,
    less_than_or_equal_to_scalar,
    less_than_less_than_or_equal_to_scalar_with_mask,
    GxB_LE_THUNK
);

macro_rules! implement_selector_with_zero {
    ($method_name:ident, $method_name_with_mask:ident, $graphblas_operator:ident) => {
        impl<T: ValueType> VectorSelector<T> {
            pub fn $method_name(
                &self,
                argument: &SparseVector<T>,
                product: &mut SparseVector<T>,
            ) -> Result<(), SparseLinearAlgebraError> {
                let context = product.context();

                context.call(|| unsafe {
                    GxB_Vector_select(
                        product.graphblas_vector(),
                        ptr::null_mut(),
                        self.accumulator,
                        $graphblas_operator,
                        argument.graphblas_vector(),
                        ptr::null_mut(),
                        self.options,
                    )
                })?;

                Ok(())
            }

            pub fn $method_name_with_mask<
                MaskValueType: ValueType,
                AsBool: AsBoolean<MaskValueType>,
            >(
                &self,
                argument: &SparseVector<T>,
                product: &mut SparseVector<T>,
                mask: &VectorMask<MaskValueType, AsBool>,
            ) -> Result<(), SparseLinearAlgebraError> {
                let context = product.context();

                context.call(|| unsafe {
                    GxB_Vector_select(
                        product.graphblas_vector(),
                        mask.graphblas_vector(),
                        self.accumulator,
                        $graphblas_operator,
                        argument.graphblas_vector(),
                        ptr::null_mut(),
                        self.options,
                    )
                })?;

                Ok(())
            }
        }
    };
}

implement_selector_with_zero!(non_zero, non_zero_with_mask, GxB_NONZERO);
implement_selector_with_zero!(zero, zero_with_mask, GxB_EQ_ZERO);
implement_selector_with_zero!(positive, positive_with_mask, GxB_GT_ZERO);
implement_selector_with_zero!(zero_or_positive, zero_or_positive_with_mask, GxB_GE_ZERO);
implement_selector_with_zero!(negative, negative_with_mask, GxB_LT_ZERO);
implement_selector_with_zero!(zero_or_negative, zero_or_negative_with_mask, GxB_LE_ZERO);

#[cfg(test)]
mod tests {
    use super::*;

    use crate::context::{Context, Mode};
    use crate::operators::binary_operator::First;

    use crate::value_types::sparse_vector::{
        FromVectorElementList, GetVectorElementValue, VectorElementList,
    };

    #[test]
    fn test_zero_scalar_selector() {
        let context = Context::init_ready(Mode::NonBlocking).unwrap();

        let element_list = VectorElementList::<u8>::from_element_vector(vec![
            (0, 1).into(),
            (1, 2).into(),
            (2, 3).into(),
            (3, 4).into(),
        ]);

        let vector_length: usize = 4;
        let vector = SparseVector::<u8>::from_element_list(
            &context.clone(),
            &vector_length,
            &element_list,
            &First::<u8, u8, u8>::new(),
        )
        .unwrap();

        let mut product_vector = SparseVector::<u8>::new(&context, &vector_length).unwrap();

        let selector = VectorSelector::new(&OperatorOptions::new_default(), None);

        selector.positive(&vector, &mut product_vector).unwrap();

        println!("{}", product_vector);

        assert_eq!(product_vector.number_of_stored_elements().unwrap(), 4);
        assert_eq!(product_vector.get_element_value(&0).unwrap(), 1);
        assert_eq!(product_vector.get_element_value(&1).unwrap(), 2);
        assert_eq!(product_vector.get_element_value(&2).unwrap(), 3);
        assert_eq!(product_vector.get_element_value(&3).unwrap(), 4);

        selector.negative(&vector, &mut product_vector).unwrap();

        println!("{}", product_vector);

        assert_eq!(product_vector.number_of_stored_elements().unwrap(), 0);
        assert_eq!(product_vector.get_element_value(&0).unwrap(), 0);
        assert_eq!(product_vector.get_element_value(&1).unwrap(), 0);
        assert_eq!(product_vector.get_element_value(&2).unwrap(), 0);
        assert_eq!(product_vector.get_element_value(&3).unwrap(), 0);
    }

    #[test]
    fn test_scalar_vector_selector() {
        let context = Context::init_ready(Mode::NonBlocking).unwrap();

        let element_list = VectorElementList::<u8>::from_element_vector(vec![
            (0, 1).into(),
            (1, 2).into(),
            (2, 3).into(),
            (3, 4).into(),
        ]);

        let vector_length: usize = 4;
        let vector = SparseVector::<u8>::from_element_list(
            &context.clone(),
            &vector_length,
            &element_list,
            &First::<u8, u8, u8>::new(),
        )
        .unwrap();

        let mut product_vector = SparseVector::<u8>::new(&context, &vector_length).unwrap();

        let selector = VectorSelector::new(&OperatorOptions::new_default(), None);

        selector
            .greater_than_scalar(&vector, &mut product_vector, &1)
            .unwrap();

        println!("{}", product_vector);

        assert_eq!(product_vector.number_of_stored_elements().unwrap(), 3);
        assert_eq!(product_vector.get_element_value(&0).unwrap(), 0);
        assert_eq!(product_vector.get_element_value(&1).unwrap(), 2);
        assert_eq!(product_vector.get_element_value(&2).unwrap(), 3);
        assert_eq!(product_vector.get_element_value(&3).unwrap(), 4);

        selector
            .less_than_scalar(&vector, &mut product_vector, &3)
            .unwrap();

        println!("{}", product_vector);

        assert_eq!(product_vector.number_of_stored_elements().unwrap(), 2);
        assert_eq!(product_vector.get_element_value(&0).unwrap(), 1);
        assert_eq!(product_vector.get_element_value(&1).unwrap(), 2);
        assert_eq!(product_vector.get_element_value(&2).unwrap(), 0);
        assert_eq!(product_vector.get_element_value(&3).unwrap(), 0);
    }
}
