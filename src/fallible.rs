use std::fmt::Debug;
use std::ops::{ControlFlow, Try};

#[derive(Debug)]
pub struct FallibleDevice<T, E> {
    dev: Fallible<T, E>,
    name: &'static str,
    has_failed: bool,
}

impl<T, E> FallibleDevice<T, E>
where
    E: Debug,
{
    pub fn new<R>(dev: R, name: &'static str) -> FallibleDevice<T, E>
    where
        R: Try<Output = T, Residual = E>,
    {
        let (has_failed, dev) = match dev.branch() {
            ControlFlow::Continue(d) => (false, Fallible::Working(d)),
            ControlFlow::Break(e) => {
                eprintln!("Fallible Device '{}' had fault: {:#?}!", name, e);
                (true, Fallible::Failed(e))
            }
        };

        FallibleDevice {
            dev,
            name,
            has_failed,
        }
    }

    pub fn run<V, F, R>(&mut self, f: F) -> Option<V>
    where
        R: Try<Output = V, Residual = E>,
        F: FnOnce(&mut T) -> R,
    {
        if self.has_failed {
            return None;
        }

        let ret = self.dev.run_fallible(f);

        match self.dev.prior_fault() {
            None => (),
            Some(e) => {
                eprintln!("Fallible Device '{}' had fault: {:#?}!", self.name, e);
                self.has_failed = true;
            }
        }

        ret
    }

    pub fn run_panic<V, F, R>(&mut self, f: F) -> Option<V>
    where
        R: Try<Output = V, Residual = E>,
        F: FnOnce(&mut T) -> R,
    {
        if self.has_failed {
            return None;
        }

        let ret = self.dev.run_fallible_panic(f);

        match self.dev.prior_fault() {
            None => (),
            Some(e) => {
                eprintln!("Fallible Device '{}' had fault: {:#?}!", self.name, e);
                self.has_failed = true;
            }
        }

        ret
    }
}

/// Wrapper type for other types which have fallible actions.
#[derive(Debug)]
pub enum Fallible<T, E> {
    Working(T),
    Failed(E),
}

impl<T, E> Fallible<T, E> {
    /// Run an action which can fail (i.e. returns an implementer of [`Try`]) recording the failure
    /// state internally and returning the "good" result if produced.
    pub fn run_fallible<V, F, R>(&mut self, f: F) -> Option<V>
    where
        R: Try<Output = V, Residual = E>,
        F: FnOnce(&mut T) -> R,
    {
        match self {
            Fallible::Working(dev) => match f(dev).branch() {
                ControlFlow::Continue(ret) => Some(ret),
                ControlFlow::Break(err) => {
                    *self = Fallible::Failed(err);
                    None
                }
            },

            Fallible::Failed(_) => None,
        }
    }

    pub fn run_fallible_panic<V, F, R>(&mut self, f: F) -> Option<V>
    where
        R: Try<Output = V, Residual = E>,
        F: FnOnce(&mut T) -> R,
        E: Debug,
    {
        match self {
            Fallible::Working(dev) => match f(dev).branch() {
                ControlFlow::Continue(ret) => Some(ret),
                ControlFlow::Break(err) => {
                    panic!("Fallible `run_fallible_panic`: {:#?}!", err)
                }
            },

            Fallible::Failed(_) => panic!("Failed called `run_fallible_panic`"),
        }
    }

    /// Get the last "fault", or returned [`Residual`] from the last run fallible action.
    ///
    /// [`Residual`]: Try::Residual
    pub fn prior_fault(&self) -> Option<&E> {
        match self {
            Fallible::Working(_) => None,
            Fallible::Failed(err) => Some(err),
        }
    }
}
