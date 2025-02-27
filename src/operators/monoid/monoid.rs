use std::marker::PhantomData;

use crate::bindings_to_graphblas_implementation::*;

use crate::value_types::value_type::ValueType;

pub trait Monoid<T>
where
    T: ValueType,
{
    fn graphblas_type(&self) -> GrB_Monoid;
}

macro_rules! implement_monoid_operator {
    ($operator_name:ident,
        $graphblas_operator_name:ident,
        $value_type:ty
    ) => {
        impl Monoid<$value_type> for $operator_name<$value_type> {
            fn graphblas_type(&self) -> GrB_Monoid {
                unsafe { $graphblas_operator_name }
            }
        }

        impl $operator_name<$value_type> {
            pub fn new() -> Self {
                $operator_name {
                    _value_type: PhantomData,
                }
            }
        }
    };
}

#[derive(Debug, Clone)]
pub struct Min<T: ValueType> {
    _value_type: PhantomData<T>,
}

implement_monoid_operator!(Min, GrB_MIN_MONOID_INT8, i8);
implement_monoid_operator!(Min, GrB_MIN_MONOID_INT16, i16);
implement_monoid_operator!(Min, GrB_MIN_MONOID_INT32, i32);
implement_monoid_operator!(Min, GrB_MIN_MONOID_INT64, i64);
implement_monoid_operator!(Min, GrB_MIN_MONOID_UINT8, u8);
implement_monoid_operator!(Min, GrB_MIN_MONOID_UINT16, u16);
implement_monoid_operator!(Min, GrB_MIN_MONOID_UINT32, u32);
implement_monoid_operator!(Min, GrB_MIN_MONOID_UINT64, u64);
implement_monoid_operator!(Min, GrB_MIN_MONOID_FP32, f32);
implement_monoid_operator!(Min, GrB_MIN_MONOID_FP64, f64);

#[derive(Debug, Clone)]
pub struct Max<T: ValueType> {
    _value_type: PhantomData<T>,
}

implement_monoid_operator!(Max, GrB_MAX_MONOID_INT8, i8);
implement_monoid_operator!(Max, GrB_MAX_MONOID_INT16, i16);
implement_monoid_operator!(Max, GrB_MAX_MONOID_INT32, i32);
implement_monoid_operator!(Max, GrB_MAX_MONOID_INT64, i64);
implement_monoid_operator!(Max, GrB_MAX_MONOID_UINT8, u8);
implement_monoid_operator!(Max, GrB_MAX_MONOID_UINT16, u16);
implement_monoid_operator!(Max, GrB_MAX_MONOID_UINT32, u32);
implement_monoid_operator!(Max, GrB_MAX_MONOID_UINT64, u64);
implement_monoid_operator!(Max, GrB_MAX_MONOID_FP32, f32);
implement_monoid_operator!(Max, GrB_MAX_MONOID_FP64, f64);

#[derive(Debug, Clone)]
pub struct Plus<T: ValueType> {
    _value_type: PhantomData<T>,
}

implement_monoid_operator!(Plus, GrB_PLUS_MONOID_INT8, i8);
implement_monoid_operator!(Plus, GrB_PLUS_MONOID_INT16, i16);
implement_monoid_operator!(Plus, GrB_PLUS_MONOID_INT32, i32);
implement_monoid_operator!(Plus, GrB_PLUS_MONOID_INT64, i64);
implement_monoid_operator!(Plus, GrB_PLUS_MONOID_UINT8, u8);
implement_monoid_operator!(Plus, GrB_PLUS_MONOID_UINT16, u16);
implement_monoid_operator!(Plus, GrB_PLUS_MONOID_UINT32, u32);
implement_monoid_operator!(Plus, GrB_PLUS_MONOID_UINT64, u64);
implement_monoid_operator!(Plus, GrB_PLUS_MONOID_FP32, f32);
implement_monoid_operator!(Plus, GrB_PLUS_MONOID_FP64, f64);

#[derive(Debug, Clone)]
pub struct Times<T: ValueType> {
    _value_type: PhantomData<T>,
}

implement_monoid_operator!(Times, GrB_TIMES_MONOID_INT8, i8);
implement_monoid_operator!(Times, GrB_TIMES_MONOID_INT16, i16);
implement_monoid_operator!(Times, GrB_TIMES_MONOID_INT32, i32);
implement_monoid_operator!(Times, GrB_TIMES_MONOID_INT64, i64);
implement_monoid_operator!(Times, GrB_TIMES_MONOID_UINT8, u8);
implement_monoid_operator!(Times, GrB_TIMES_MONOID_UINT16, u16);
implement_monoid_operator!(Times, GrB_TIMES_MONOID_UINT32, u32);
implement_monoid_operator!(Times, GrB_TIMES_MONOID_UINT64, u64);
implement_monoid_operator!(Times, GrB_TIMES_MONOID_FP32, f32);
implement_monoid_operator!(Times, GrB_TIMES_MONOID_FP64, f64);

#[derive(Debug, Clone)]
pub struct Any<T: ValueType> {
    _value_type: PhantomData<T>,
}

implement_monoid_operator!(Any, GxB_ANY_BOOL_MONOID, bool);
implement_monoid_operator!(Any, GxB_ANY_INT8_MONOID, i8);
implement_monoid_operator!(Any, GxB_ANY_INT16_MONOID, i16);
implement_monoid_operator!(Any, GxB_ANY_INT32_MONOID, i32);
implement_monoid_operator!(Any, GxB_ANY_INT64_MONOID, i64);
implement_monoid_operator!(Any, GxB_ANY_UINT8_MONOID, u8);
implement_monoid_operator!(Any, GxB_ANY_UINT16_MONOID, u16);
implement_monoid_operator!(Any, GxB_ANY_UINT32_MONOID, u32);
implement_monoid_operator!(Any, GxB_ANY_UINT64_MONOID, u64);
implement_monoid_operator!(Any, GxB_ANY_FP32_MONOID, f32);
implement_monoid_operator!(Any, GxB_ANY_FP64_MONOID, f64);

#[derive(Debug, Clone)]
pub struct LogicalOr<T: ValueType> {
    _value_type: PhantomData<T>,
}

#[derive(Debug, Clone)]
pub struct LogicalAnd<T: ValueType> {
    _value_type: PhantomData<T>,
}

#[derive(Debug, Clone)]
pub struct LogicalExclusiveOr<T: ValueType> {
    _value_type: PhantomData<T>,
}

#[derive(Debug, Clone)]
pub struct Equal<T: ValueType> {
    _value_type: PhantomData<T>,
}

implement_monoid_operator!(LogicalOr, GrB_LOR_MONOID_BOOL, bool);
implement_monoid_operator!(LogicalAnd, GrB_LAND_MONOID_BOOL, bool);
implement_monoid_operator!(LogicalExclusiveOr, GrB_LXOR_MONOID_BOOL, bool);
implement_monoid_operator!(Equal, GrB_LXNOR_MONOID_BOOL, bool);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_binary_operator() {
        let min_monoid = Min::<f32>::new();
        let _graphblas_type = min_monoid.graphblas_type();
    }
}

/*
    //--------------------------------------------------------------------------
    // 10 MIN monoids: (not for complex types)
    //--------------------------------------------------------------------------
    // preferred names from the v1.3 spec:
    GrB_MIN_MONOID_INT8,        // identity: INT8_MAX     terminal: INT8_MIN
    GrB_MIN_MONOID_INT16,       // identity: INT16_MAX    terminal: INT16_MIN
    GrB_MIN_MONOID_INT32,       // identity: INT32_MAX    terminal: INT32_MIN
    GrB_MIN_MONOID_INT64,       // identity: INT64_MAX    terminal: INT32_MIN
    GrB_MIN_MONOID_UINT8,       // identity: UINT8_MAX    terminal: 0
    GrB_MIN_MONOID_UINT16,      // identity: UINT16_MAX   terminal: 0
    GrB_MIN_MONOID_UINT32,      // identity: UINT32_MAX   terminal: 0
    GrB_MIN_MONOID_UINT64,      // identity: UINT64_MAX   terminal: 0
    GrB_MIN_MONOID_FP32,        // identity: INFINITY     terminal: -INFINITY
    GrB_MIN_MONOID_FP64,        // identity: INFINITY     terminal: -INFINITY

    //--------------------------------------------------------------------------
    // 10 MAX monoids:
    //--------------------------------------------------------------------------

    // preferred names from the v1.3 spec:
    GrB_MAX_MONOID_INT8,        // identity: INT8_MIN     terminal: INT8_MAX
    GrB_MAX_MONOID_INT16,       // identity: INT16_MIN    terminal: INT16_MAX
    GrB_MAX_MONOID_INT32,       // identity: INT32_MIN    terminal: INT32_MAX
    GrB_MAX_MONOID_INT64,       // identity: INT64_MIN    terminal: INT64_MAX
    GrB_MAX_MONOID_UINT8,       // identity: 0            terminal: UINT8_MAX
    GrB_MAX_MONOID_UINT16,      // identity: 0            terminal: UINT16_MAX
    GrB_MAX_MONOID_UINT32,      // identity: 0            terminal: UINT32_MAX
    GrB_MAX_MONOID_UINT64,      // identity: 0            terminal: UINT64_MAX
    GrB_MAX_MONOID_FP32,        // identity: -INFINITY    terminal: INFINITY
    GrB_MAX_MONOID_FP64,        // identity: -INFINITY    terminal: INFINITY

    //--------------------------------------------------------------------------
    // 12 PLUS monoids:
    //--------------------------------------------------------------------------

    // preferred names from the v1.3 spec:
    GrB_PLUS_MONOID_INT8,       // identity: 0
    GrB_PLUS_MONOID_INT16,      // identity: 0
    GrB_PLUS_MONOID_INT32,      // identity: 0
    GrB_PLUS_MONOID_INT64,      // identity: 0
    GrB_PLUS_MONOID_UINT8,      // identity: 0
    GrB_PLUS_MONOID_UINT16,     // identity: 0
    GrB_PLUS_MONOID_UINT32,     // identity: 0
    GrB_PLUS_MONOID_UINT64,     // identity: 0
    GrB_PLUS_MONOID_FP32,       // identity: 0
    GrB_PLUS_MONOID_FP64,       // identity: 0

    // complex monoids:
    GxB_PLUS_FC32_MONOID,       // identity: 0
    GxB_PLUS_FC64_MONOID,       // identity: 0

    //--------------------------------------------------------------------------
    // 12 TIMES monoids: identity value is 1, int* and uint* are terminal
    //--------------------------------------------------------------------------

    // preferred names from the v1.3 spec:
    GrB_TIMES_MONOID_INT8,      // identity: 1            terminal: 0
    GrB_TIMES_MONOID_INT16,     // identity: 1            terminal: 0
    GrB_TIMES_MONOID_INT32,     // identity: 1            terminal: 0
    GrB_TIMES_MONOID_INT64,     // identity: 1            terminal: 0
    GrB_TIMES_MONOID_UINT8,     // identity: 1            terminal: 0
    GrB_TIMES_MONOID_UINT16,    // identity: 1            terminal: 0
    GrB_TIMES_MONOID_UINT32,    // identity: 1            terminal: 0
    GrB_TIMES_MONOID_UINT64,    // identity: 1            terminal: 0
    GrB_TIMES_MONOID_FP32,      // identity: 1
    GrB_TIMES_MONOID_FP64,      // identity: 1

    // complex monoids:
    GxB_TIMES_FC32_MONOID,      // identity: 1
    GxB_TIMES_FC64_MONOID,      // identity: 1

    //--------------------------------------------------------------------------
    // 13 ANY monoids:
    //--------------------------------------------------------------------------

    GxB_ANY_BOOL_MONOID,        // identity: any value    terminal: any value
    GxB_ANY_INT8_MONOID,        // identity: any value    terminal: any value
    GxB_ANY_INT16_MONOID,       // identity: any value    terminal: any value
    GxB_ANY_INT32_MONOID,       // identity: any value    terminal: any value
    GxB_ANY_INT64_MONOID,       // identity: any value    terminal: any value
    GxB_ANY_UINT8_MONOID,       // identity: any value    terminal: any value
    GxB_ANY_UINT16_MONOID,      // identity: any value    terminal: any value
    GxB_ANY_UINT32_MONOID,      // identity: any value    terminal: any value
    GxB_ANY_UINT64_MONOID,      // identity: any value    terminal: any value
    GxB_ANY_FP32_MONOID,        // identity: any value    terminal: any value
    GxB_ANY_FP64_MONOID,        // identity: any value    terminal: any value
    GxB_ANY_FC32_MONOID,        // identity: any value    terminal: any value
    GxB_ANY_FC64_MONOID,        // identity: any value    terminal: any value

    //--------------------------------------------------------------------------
    // 4 Boolean monoids: (see also the GxB_ANY_BOOL_MONOID above)
    //--------------------------------------------------------------------------

    GxB_EQ_BOOL_MONOID,         // (alternative name for GrB_LXNOR_MONOID_BOOL)

    // preferred names from the v1.3 spec:
    GrB_LOR_MONOID_BOOL,        // identity: false        terminal: true
    GrB_LAND_MONOID_BOOL,       // identity: true         terminal: false
    GrB_LXOR_MONOID_BOOL,       // identity: false
    GrB_LXNOR_MONOID_BOOL,      // identity: true

    //--------------------------------------------------------------------------
    // 16 Bitwise-or monoids:
    //--------------------------------------------------------------------------

    // BOR monoids (bitwise or):
    GxB_BOR_UINT8_MONOID,       // identity: 0   terminal: 0xFF
    GxB_BOR_UINT16_MONOID,      // identity: 0   terminal: 0xFFFF
    GxB_BOR_UINT32_MONOID,      // identity: 0   terminal: 0xFFFFFFFF
    GxB_BOR_UINT64_MONOID,      // identity: 0   terminal: 0xFFFFFFFFFFFFFFFF

    // BAND monoids (bitwise and):
    GxB_BAND_UINT8_MONOID,      // identity: 0xFF               terminal: 0
    GxB_BAND_UINT16_MONOID,     // identity: 0xFFFF             terminal: 0
    GxB_BAND_UINT32_MONOID,     // identity: 0xFFFFFFFF         terminal: 0
    GxB_BAND_UINT64_MONOID,     // identity: 0xFFFFFFFFFFFFFFFF terminal: 0

    // BXOR monoids (bitwise xor):
    GxB_BXOR_UINT8_MONOID,      // identity: 0
    GxB_BXOR_UINT16_MONOID,     // identity: 0
    GxB_BXOR_UINT32_MONOID,     // identity: 0
    GxB_BXOR_UINT64_MONOID,     // identity: 0

    // BXNOR monoids (bitwise xnor):
    GxB_BXNOR_UINT8_MONOID,     // identity: 0xFF
    GxB_BXNOR_UINT16_MONOID,    // identity: 0xFFFF
    GxB_BXNOR_UINT32_MONOID,    // identity: 0xFFFFFFFF
    GxB_BXNOR_UINT64_MONOID ;   // identity: 0xFFFFFFFFFFFFFFFF

*/
