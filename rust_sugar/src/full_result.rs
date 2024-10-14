use std::{
    convert::{self, Infallible}, 
    fmt::{self, Debug}, 
    hint, 
    ops::{
        ControlFlow, 
        Deref, 
        DerefMut, 
        FromResidual, 
        Residual, 
        Try
    }
};

pub enum FullResult<T, E1, E2> {
    Ok(T),
    SoftErr(E1),
    HardErr(E2)
}

impl<T, E1, E2> FullResult<T, E1, E2> {
    pub fn is_ok(&self) -> bool {
        matches!(*self, FullResult::Ok(_))
    }

    pub fn is_ok_and(self, f: impl FnOnce(T) -> bool) -> bool {
        if let FullResult::Ok(value) = self {
            return f(value);
        }
        return false;
    }

    pub fn is_soft_err(&self) -> bool {
        matches!(*self, FullResult::SoftErr(_))
    }

    pub fn is_soft_err_and(self, f: impl FnOnce(E1) -> bool) -> bool {
        if let FullResult::SoftErr(value) = self {
            return f(value);
        }
        return false;
    }

    pub fn is_hard_err(&self) -> bool {
        matches!(*self, FullResult::HardErr(_))
    }

    pub fn is_hard_err_and(self, f: impl FnOnce(E2) -> bool) -> bool {
        if let FullResult::HardErr(value) = self {
            return f(value);
        }
        return false;
    }

    pub fn ok(self) -> Option<T> {
        if let FullResult::Ok(value) = self {
            return Some(value);
        }
        return None;
    }

    pub fn soft_err(self) -> Option<E1> {
        if let FullResult::SoftErr(value) = self {
            return Some(value);
        }
        return None;
    }

    pub fn hard_err(self) -> Option<E2> {
        if let FullResult::HardErr(value) = self {
            return Some(value);
        }
        return None;
    }

    pub const fn as_ref(&self) -> FullResult<&T, &E1, &E2> {
        return match *self {
            FullResult::Ok(ref value) => FullResult::Ok(value),
            FullResult::SoftErr(ref value) => FullResult::SoftErr(value),
            FullResult::HardErr(ref value) => FullResult::HardErr(value)
        };
    }

    pub const fn as_mut(&mut self) -> FullResult<&mut T, &mut E1, &mut E2> {
        return match *self {
            FullResult::Ok(ref mut value) => FullResult::Ok(value),
            FullResult::SoftErr(ref mut value) => FullResult::SoftErr(value),
            FullResult::HardErr(ref mut value) => FullResult::HardErr(value)
        };
    }

    pub fn map<U>(self, f: impl FnOnce(T) -> U) -> FullResult<U, E1, E2> {
        return match self {
            FullResult::Ok(value) => FullResult::Ok(f(value)),
            FullResult::SoftErr(value) => FullResult::SoftErr(value),
            FullResult::HardErr(value) => FullResult::HardErr(value)
        };
    }

    pub fn map_or<U>(self, default: U, f: impl FnOnce(T) -> U) -> U {
        if let FullResult::Ok(value) = self {
            return f(value);
        }
        return default;
    }

    pub fn map_or_else<U>(
        self, 
        default: impl FnOnce(Either<E1, E2>) -> U, 
        f: impl FnOnce(T) -> U
    ) -> U {
        return match self {
            FullResult::Ok(value) => f(value),
            FullResult::SoftErr(value) => default(Either::This(value)),
            FullResult::HardErr(value) => default(Either::That(value))
        };
    }

    pub fn map_soft_err<F1>(self, f: impl FnOnce(E1) -> F1) -> FullResult<T, F1, E2> {
        return match self {
            FullResult::Ok(value) => FullResult::Ok(value),
            FullResult::SoftErr(value) => FullResult::SoftErr(f(value)),
            FullResult::HardErr(value) => FullResult::HardErr(value)
        };
    }

    pub fn map_soft_err_or<F1>(self, default: F1, f: impl FnOnce(E1) -> F1) -> F1 {
        if let FullResult::SoftErr(value) = self {
            return f(value);
        }
        return default;
    }

    pub fn map_soft_err_or_else<F1>(
        self, 
        default: impl FnOnce(Either<T, E2>) -> F1, 
        f: impl FnOnce(E1) -> F1
    ) -> F1 {
        return match self {
            FullResult::Ok(value) => default(Either::This(value)),
            FullResult::SoftErr(value) => f(value),
            FullResult::HardErr(value) => default(Either::That(value))
        };
    }

    pub fn map_hard_err<F2>(self, f: impl FnOnce(E2) -> F2) -> FullResult<T, E1, F2> {
        return match self {
            FullResult::Ok(value) => FullResult::Ok(value),
            FullResult::SoftErr(value) => FullResult::SoftErr(value),
            FullResult::HardErr(value) => FullResult::HardErr(f(value))
        };
    }

    pub fn map_hard_err_or<F2>(self, default: F2, f: impl FnOnce(E2) -> F2) -> F2 {
        if let FullResult::HardErr(value) = self {
            return f(value);
        }
        return default;
    }

    pub fn map_hard_err_or_else<F2>(
        self, 
        default: impl FnOnce(Either<T, E1>) -> F2,
        f: impl FnOnce(E2) -> F2
    ) -> F2 {
        return match self {
            FullResult::Ok(value) => default(Either::This(value)),
            FullResult::SoftErr(value) => default(Either::That(value)),
            FullResult::HardErr(value) => f(value)
        };
    }

    pub fn inspect(self, f: impl FnOnce(&T)) -> Self {
        if let FullResult::Ok(ref t) = self {
            f(t);
        }
        return self;
    }

    pub fn inspect_soft_err(self, f: impl FnOnce(&E1)) -> Self {
        if let FullResult::SoftErr(ref se) = self {
            f(se);
        }
        return self;
    }

    pub fn inspect_hard_err(self, f: impl FnOnce(&E2)) -> Self {
        if let FullResult::HardErr(ref he) = self {
            f(he);
        }
        return self;
    }

    pub fn as_deref(&self) -> FullResult<
        &<T as Deref>::Target, 
        &E1, &E2
    > where 
        T : Deref 
    {
        self.as_ref().map(|t| t.deref())
    }

    pub fn as_deref_mut(&mut self) -> FullResult<
        &mut <T as Deref>::Target, 
        &mut E1, &mut E2
    > where 
        T : DerefMut 
    {
        self.as_mut().map(|t| t.deref_mut())
    }

    

    #[inline(always)]
    #[track_caller]
    pub fn expect(self, msg: &str) -> T 
    where 
        E1 : Debug,
        E2 : Debug
    {
        match self {
            FullResult::Ok(t) => t,
            FullResult::SoftErr(se) => unwrap_failed(msg, &se),
            FullResult::HardErr(he) => unwrap_failed(msg, &he)
        }
    }

    #[inline(always)]
    #[track_caller]
    pub fn expect_soft_err(self, msg: &str) -> E1 
    where 
        T : Debug,
        E2 : Debug
    {
        match self {
            FullResult::Ok(t) => unwrap_failed(msg, &t),
            FullResult::SoftErr(se) => se,
            FullResult::HardErr(he) => unwrap_failed(msg, &he)
        }
    }

    #[inline(always)]
    #[track_caller]
    pub fn expect_hard_err(self, msg: &str) -> E2
    where 
        T : Debug,
        E1 : Debug
    {
        match self {
            FullResult::Ok(t) => unwrap_failed(msg, &t),
            FullResult::SoftErr(se) => unwrap_failed(msg, &se),
            FullResult::HardErr(he) => he
        }
    }

    #[inline(always)]
    #[track_caller]
    pub fn unwrap(self) -> T
    where
        E1 : Debug,
        E2 : Debug
    {
        match self {
            FullResult::Ok(t) => t,
            FullResult::SoftErr(se) => unwrap_failed(
                "called `FullResult::unwrap()` on an `SoftErr` value", &se
            ),
            FullResult::HardErr(he) => unwrap_failed(
                "called `FullResult::unwrap()` on an `HardErr` value", &he
            )
        }
    }

    #[inline(always)]
    #[track_caller]
    pub fn unwrap_soft_err(self) -> E1
    where
        T : Debug,
        E2 : Debug
    {
        match self {
            FullResult::Ok(t) => unwrap_failed(
                "called `FullResult::unwrap_soft_err()` on an `Ok` value", &t
            ),
            FullResult::SoftErr(se) => se,
            FullResult::HardErr(he) => unwrap_failed(
                "called `FullResult::unwrap_soft_err()` on an `HardErr` value", &he
            )
        }
    }

    #[inline(always)]
    #[track_caller]
    pub fn unwrap_hard_err(self) -> E2
    where
        T : Debug,
        E1 : Debug
    {
        match self {
            FullResult::Ok(t) => unwrap_failed(
                "called `FullResult::unwrap_hard_err()` on an `Ok` value", &t
            ),
            FullResult::SoftErr(se) => unwrap_failed(
                "called `FullResult::unwrap_hard_err()` on an `SoftErr` value", &se
            ),
            FullResult::HardErr(he) => he,
        }
    }

    #[inline]
    pub fn unwrap_or(self, default: T) -> T {
        if let FullResult::Ok(t) = self {
            return t;
        }
        return default;
    }

    #[inline]
    pub fn unwrap_or_else(self, f: impl FnOnce(Either<E1, E2>) -> T) -> T {
        match self {
            FullResult::Ok(t) => t,
            FullResult::SoftErr(se) => f(Either::This(se)),
            FullResult::HardErr(he) => f(Either::That(he))
        }
    }

    #[inline]
    pub fn unwrap_or_default(self) -> T
    where 
        T : Default
    {
        if let FullResult::Ok(t) = self {
            return t;
        }
        return T::default();
    }

    #[inline]
    pub fn into_ok(self) -> T
    where
        E1 : Into<!>,
        E2 : Into<!>
    {
        match self {
            FullResult::Ok(t) => t,
            FullResult::SoftErr(se) => se.into(),
            FullResult::HardErr(he) => he.into(),
        }
    }

    #[inline]
    pub fn into_soft_err(self) -> E1
    where
        T : Into<!>,
        E2 : Into<!>
    {
        match self {
            FullResult::Ok(t) => t.into(),
            FullResult::SoftErr(se) => se,
            FullResult::HardErr(he) => he.into(),
        }
    }

    #[inline]
    pub fn into_hard_err(self) -> E2
    where
        T : Into<!>,
        E1 : Into<!>
    {
        match self {
            FullResult::Ok(t) => t.into(),
            FullResult::SoftErr(se) => se.into(),
            FullResult::HardErr(he) => he
        }
    }

    pub fn and<U>(self, fres: FullResult<U, E1, E2>) -> FullResult<U, E1, E2> {
        match (self, fres) {
            (FullResult::Ok(_), output) => output,
            (FullResult::HardErr(he), _) 
                | (_, FullResult::HardErr(he)) 
            => FullResult::HardErr(he),
            (FullResult::SoftErr(se), _) => FullResult::SoftErr(se)
        }
    }

    pub fn and_then<U>(self, op: impl FnOnce(T) -> FullResult<U, E1, E2>) -> FullResult<U, E1, E2> {
        match self {
            FullResult::Ok(t) => op(t),
            FullResult::SoftErr(se) => FullResult::SoftErr(se),
            FullResult::HardErr(he) => FullResult::HardErr(he),
        }
    }

    pub fn or<F1>(self, fres: FullResult<T, F1, E2>) -> FullResult<T, F1, E2> {
        match (self, fres) {
            (FullResult::Ok(t), _) => FullResult::Ok(t),
            (FullResult::HardErr(he), _) 
                | (_, FullResult::HardErr(he)) 
            => FullResult::HardErr(he),
            (FullResult::SoftErr(_), output) => output
        }
    }

    pub fn or_else<F1>(self, op: impl FnOnce(Either<E1, E2>) -> FullResult<T, F1, E2>) -> FullResult<T, F1, E2> {
        match self {
            FullResult::Ok(t) => FullResult::Ok(t),
            FullResult::SoftErr(se) => op(Either::This(se)),
            FullResult::HardErr(he) => op(Either::That(he)),
        }
    }

    #[inline]
    #[track_caller]
    pub unsafe fn unwrap_unchecked(self) -> T {
        debug_assert!(self.is_ok());
        let FullResult::Ok(t) = self else {
            unsafe { hint::unreachable_unchecked() }
        };
        return t;
    }

    #[inline]
    #[track_caller]
    pub unsafe fn unwrap_soft_err_unchecked(self) -> E1 {
        debug_assert!(self.is_soft_err());
        let FullResult::SoftErr(se) = self else {
            unsafe { hint::unreachable_unchecked() }
        };
        return se;
    }

    #[inline]
    #[track_caller]
    pub unsafe fn unwrap_hard_err_unchecked(self) -> E2 {
        debug_assert!(self.is_hard_err());
        let FullResult::HardErr(he) = self else {
            unsafe { hint::unreachable_unchecked() }
        };
        return he;
    }
}

impl<T, E1, E2> FullResult<&T, E1, E2> {
    pub fn copied(self) -> FullResult<T, E1, E2> where T : Copy {
        self.map(|&t| t)
    }

    pub fn cloned(self) -> FullResult<T, E1, E2> where T : Clone {
        self.map(|t| t.clone())
    }
}

impl<T, E1, E2> FullResult<&mut T, E1, E2> {
    pub fn copied(self) -> FullResult<T, E1, E2> where T : Copy {
        self.map(|&mut t| t)
    }

    pub fn cloned(self) -> FullResult<T, E1, E2> where T : Clone {
        self.map(|t| t.clone())
    }
}

impl<T, E1, E2> FullResult<Option<T>, E1, E2> {
    pub fn transpose(self) -> Option<FullResult<T, E1, E2>> {
        match self {
            FullResult::Ok(Some(t)) => Some(FullResult::Ok(t)),
            FullResult::Ok(None) => None,
            FullResult::SoftErr(se) => Some(FullResult::SoftErr(se)),
            FullResult::HardErr(he) => Some(FullResult::HardErr(he))
        }
    }
}

impl<T, E1, E2> FullResult<FullResult<T, E1, E2>, E1, E2> {
    pub fn flatten(self) -> FullResult<T, E1, E2> {
        self.and_then(convert::identity)
    }
}

impl<T, E1, E2> FullResult<Result<T, E1>, E1, E2> {
    pub fn flatten_soft(self) -> FullResult<T, E1, E2> {
        self.and_then(|res| match res {
            Ok(t) => FullResult::Ok(t),
            Err(e) => FullResult::SoftErr(e)
        })
    }
}

impl<T, E1, E2> FullResult<Result<T, E2>, E1, E2> {
    pub fn flatten_hard(self) -> FullResult<T, E1, E2> {
        self.and_then(|res| match res {
            Ok(t) => FullResult::Ok(t),
            Err(e) => FullResult::HardErr(e)
        })
    }
}

impl<T, E1, E2> Clone for FullResult<T, E1, E2>
where
    T : Clone,
    E1 : Clone,
    E2 : Clone,
{
    #[inline]
    fn clone(&self) -> Self {
        match self {
            FullResult::Ok(x) => FullResult::Ok(x.clone()),
            FullResult::SoftErr(x) => FullResult::SoftErr(x.clone()),
            FullResult::HardErr(x) => FullResult::HardErr(x.clone()),
        }
    }

    #[inline]
    fn clone_from(&mut self, source: &Self) {
        match (self, source) {
            (FullResult::Ok(to), FullResult::Ok(from)) => to.clone_from(from),
            (FullResult::SoftErr(to), FullResult::SoftErr(from)) => to.clone_from(from),
            (FullResult::HardErr(to), FullResult::HardErr(from)) => to.clone_from(from),
            (to, from) => *to = from.clone(),
        }
    }
}

pub trait OptionToFullResult<T> {
    fn ok_or_soft<E1, E2>(self, se: E1) -> FullResult<T, E1, E2>;

    fn ok_or_hard<E1, E2>(self, he: E2) -> FullResult<T, E1, E2>;
}

impl<T> OptionToFullResult<T> for Option<T> {
    fn ok_or_soft<E1, E2>(self, se: E1) -> FullResult<T, E1, E2> {
        if let Some(t) = self {
            return FullResult::Ok(t);
        }
        return FullResult::SoftErr(se);
    }

    fn ok_or_hard<E1, E2>(self, he: E2) -> FullResult<T, E1, E2> {
        if let Some(t) = self {
            return FullResult::Ok(t);
        }
        return FullResult::HardErr(he);
    }
}

pub trait ResultToFullResult<T, E> {
    fn soften<E2>(self) -> FullResult<T, E, E2>;
    fn map_soften<F, E2>(self, f: impl FnOnce(E) -> F) -> FullResult<T, F, E2>;
    fn harden<E1>(self) -> FullResult<T, E1, E>;
    fn map_harden<E1, F>(self, f: impl FnOnce(E) -> F) -> FullResult<T, E1, F>;
}

impl<T, E> ResultToFullResult<T, E> for Result<T, E> {
    fn soften<E2>(self) -> FullResult<T, E, E2> {
        match self {
            Ok(t) => FullResult::Ok(t),
            Err(e) => FullResult::SoftErr(e)
        }
    }

    fn map_soften<F, E2>(self, f: impl FnOnce(E) -> F) -> FullResult<T, F, E2> {
        match self {
            Ok(t) => FullResult::Ok(t),
            Err(e) => FullResult::SoftErr(f(e))
        }
    }

    fn harden<E1>(self) -> FullResult<T, E1, E> {
        match self {
            Ok(t) => FullResult::Ok(t),
            Err(e) => FullResult::HardErr(e)
        }
    }

    fn map_harden<E1, F>(self, f: impl FnOnce(E) -> F) -> FullResult<T, E1, F> {
        match self {
            Ok(t) => FullResult::Ok(t),
            Err(e) => FullResult::HardErr(f(e))
        }
    }
}

impl<T, E1, E2> Try for FullResult<T, E1, E2>
where 
    FullResult<T, E1, E2> : FromResidual<FullResult<Infallible, E1, E2>> 
{
    type Output = T;
    type Residual = FullResult<Infallible, E1, E2>;

    fn branch(self) -> ControlFlow<Self::Residual, Self::Output> {
        return match self {
            FullResult::Ok(output) => ControlFlow::Continue(output),
            FullResult::SoftErr(err) => ControlFlow::Break(FullResult::SoftErr(err)),
            FullResult::HardErr(err) => ControlFlow::Break(FullResult::HardErr(err))
        };
    }

    fn from_output(output: Self::Output) -> Self {
        Self::Ok(output)
    }
}

impl<T, E1, E2, F1: From<E1>, F2: From<E2>> 
    FromResidual<FullResult<Infallible, E1, E2>> 
for FullResult<T, F1, F2> {
    fn from_residual(residual: FullResult<Infallible, E1, E2>) -> Self {
        match residual {
            FullResult::SoftErr(se) => FullResult::SoftErr(From::from(se)),
            FullResult::HardErr(he) => FullResult::HardErr(From::from(he)),
        }
    }
}

impl<T, E1, E2> Residual<T> for FullResult<Infallible, E1, E2> {
    type TryType = FullResult<T, E1, E2>;
}

pub enum Either<T, U> {
    This(T), That(U)
}

#[inline(never)]
#[cold]
#[track_caller]
fn unwrap_failed(msg: &str, error: &dyn fmt::Debug) -> ! {
    panic!("{msg}: {error:?}")
}