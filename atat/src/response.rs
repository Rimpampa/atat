use crate::{Error, InternalError, NoCustomError};
use heapless::Vec;

#[derive(Debug, Clone, PartialEq)]
pub enum Response<const N: usize, E = NoCustomError> {
    Ok(Vec<u8, N>),
    Prompt(u8),
    Error(Error<E>),
}

impl<const N: usize, E> Response<N, E> {
    pub fn ok(value: &[u8]) -> Self {
        Response::Ok(Vec::from_slice(value).unwrap())
    }
}

impl<const N: usize, E> Default for Response<N, E> {
    fn default() -> Self {
        Response::Ok(Vec::new())
    }
}

impl<'a, const N: usize, E: From<&'a [u8]>> From<Result<&'a [u8], InternalError<'a>>>
    for Response<N, E>
{
    fn from(value: Result<&'a [u8], InternalError<'a>>) -> Self {
        match value {
            Ok(slice) => Response::Ok(Vec::from_slice(slice).unwrap()),
            Err(error) => error.into(),
        }
    }
}

impl<'a, const N: usize, E: From<&'a [u8]>> From<InternalError<'a>> for Response<N, E> {
    fn from(v: InternalError<'a>) -> Self {
        Response::Error(v.parse())
    }
}

impl<'a, const N: usize, E: From<&'a [u8]>> From<&'a Response<N, E>>
    for Result<&'a [u8], &'a Error<E>>
{
    fn from(value: &'a Response<N, E>) -> Self {
        match value {
            Response::Ok(slice) => Ok(slice),
            Response::Prompt(_) => Ok(&[]),
            Response::Error(e) => Err(e),
        }
    }
}
