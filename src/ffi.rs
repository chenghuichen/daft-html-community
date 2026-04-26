use std::sync::Arc;

use arrow::{
    array::{cast::AsArray, Array, LargeStringBuilder, ListBuilder},
    datatypes::{DataType, Field},
};
use daft_ext::prelude::*;

pub(crate) fn decode_input(data: ArrowData) -> DaftResult<(Arc<dyn Array>, Field)> {
    let ffi_schema: arrow::ffi::FFI_ArrowSchema = data.schema.into();
    let field = Field::try_from(&ffi_schema)?;
    let ffi_array: arrow::ffi::FFI_ArrowArray = data.array.into();
    let array_data = unsafe { arrow::ffi::from_ffi(ffi_array, &ffi_schema) }?;
    Ok((arrow::array::make_array(array_data), field))
}

pub(crate) fn encode_output(
    array_data: &arrow::array::ArrayData,
    field: &Field,
) -> DaftResult<ArrowData> {
    let (out_arr, _) = arrow::ffi::to_ffi(array_data)?;
    let out_ffi_schema = arrow::ffi::FFI_ArrowSchema::try_from(field)?;
    Ok(ArrowData {
        array: out_arr.into(),
        schema: unsafe { ArrowSchema::from_owned(out_ffi_schema) },
    })
}

pub(crate) fn require_string_arg(
    args: &[ArrowSchema],
    pos: usize,
    fn_name: &str,
) -> DaftResult<Field> {
    let field = Field::try_from(&args[pos])?;
    match field.data_type() {
        DataType::Utf8 | DataType::LargeUtf8 => Ok(field),
        dt => Err(DaftError::TypeError(format!(
            "{fn_name}: argument {pos} must be String, got {dt:?}"
        ))),
    }
}

pub(crate) fn scalar_string(array: &Arc<dyn Array>, arg_name: &str) -> DaftResult<String> {
    match array.data_type() {
        DataType::Utf8 => {
            let arr = array.as_string::<i32>();
            if arr.is_null(0) {
                Err(DaftError::RuntimeError(format!(
                    "{arg_name} cannot be null"
                )))
            } else {
                Ok(arr.value(0).to_string())
            }
        }
        DataType::LargeUtf8 => {
            let arr = array.as_string::<i64>();
            if arr.is_null(0) {
                Err(DaftError::RuntimeError(format!(
                    "{arg_name} cannot be null"
                )))
            } else {
                Ok(arr.value(0).to_string())
            }
        }
        dt => Err(DaftError::RuntimeError(format!(
            "{arg_name} must be a string, got {dt:?}"
        ))),
    }
}

pub(crate) fn apply_string_map(
    input: &Arc<dyn Array>,
    len: usize,
    mut f: impl FnMut(&str) -> Option<String>,
    builder: &mut LargeStringBuilder,
) -> DaftResult<()> {
    match input.data_type() {
        DataType::Utf8 => {
            let arr = input.as_string::<i32>();
            for i in 0..len {
                if arr.is_null(i) {
                    builder.append_null();
                } else {
                    match f(arr.value(i)) {
                        Some(s) => builder.append_value(s),
                        None => builder.append_null(),
                    }
                }
            }
        }
        DataType::LargeUtf8 => {
            let arr = input.as_string::<i64>();
            for i in 0..len {
                if arr.is_null(i) {
                    builder.append_null();
                } else {
                    match f(arr.value(i)) {
                        Some(s) => builder.append_value(s),
                        None => builder.append_null(),
                    }
                }
            }
        }
        dt => {
            return Err(DaftError::RuntimeError(format!(
                "expected String/LargeUtf8, got {dt:?}"
            )))
        }
    }
    Ok(())
}

pub(crate) fn apply_list_map(
    input: &Arc<dyn Array>,
    len: usize,
    mut f: impl FnMut(&str) -> Vec<String>,
    builder: &mut ListBuilder<LargeStringBuilder>,
) -> DaftResult<()> {
    match input.data_type() {
        DataType::Utf8 => {
            let arr = input.as_string::<i32>();
            for i in 0..len {
                if arr.is_null(i) {
                    builder.append_null();
                } else {
                    for item in f(arr.value(i)) {
                        builder.values().append_value(item);
                    }
                    builder.append(true);
                }
            }
        }
        DataType::LargeUtf8 => {
            let arr = input.as_string::<i64>();
            for i in 0..len {
                if arr.is_null(i) {
                    builder.append_null();
                } else {
                    for item in f(arr.value(i)) {
                        builder.values().append_value(item);
                    }
                    builder.append(true);
                }
            }
        }
        dt => {
            return Err(DaftError::RuntimeError(format!(
                "expected String/LargeUtf8, got {dt:?}"
            )))
        }
    }
    Ok(())
}
