

pub trait Resultable<T> where T : Clone {
    fn to_result_with_error<E>(&self, err: E) -> Result<T, E>;
}

impl<T> Resultable<T> for Option<T> where T : Clone {
    fn to_result_with_error<E>(&self, err: E) -> Result<T, E> {
        return match self {
            Some(ok) => Ok(ok.clone()),
            None => Err(err)
        }
    }
}