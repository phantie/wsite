pub use super::super::{Error, *};

pub fn op_result(result: std::result::Result<NamedRows, miette::Report>) -> OpResult {
    use itertools::Itertools;
    let result = result.map_err(Error::EngineError)?;

    let headers = result.headers.iter().map(String::as_str).collect_vec();
    let rows = result.rows.iter().map(Vec::as_slice).collect_vec();

    match (&headers[..], &rows[..]) {
        (["status"], [[v]]) if v == &DataValue::from("OK") => Ok(()),
        _ => Err(Error::ResultError(result)),
    }
}
