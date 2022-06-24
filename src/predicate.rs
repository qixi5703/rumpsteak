use std::marker::PhantomData;
use std::collections::HashMap;
use std::fmt::Debug;
use std::convert::TryFrom;

use pest::Parser;
use pest_derive::Parser;
use pest::iterators::Pair;
use pest::error::Error;
use std::cmp::Ord;
use std::hash::Hash;

/// A `Predicate` is a structure that allows to express some properties on some
/// variables. The trait contains a single function which assigns a truth value
/// to a set of variables.
/// A Predicate must implement the `Default` so that it can be instantiated by
/// the system.
/// [min_const_generics](https://github.com/rust-lang/rust/issues/74878) help to
/// implement such types in our context.
pub trait Predicate: Default {
    type Name;
    type Value;
    type Error: Debug;

    /// This function checks whether the predicate holds on the current values
    /// of variables.
    /// It returns a `Result` which is `Ok(())` if the predicate holds, or an
    /// arbitrary error (of type `E`) if not. 
    /// Most likely, one may want to have `()` as the error type (falling back
    /// on something simili-boolean), but having this generic type allow more
    /// precise analysis in case of failure (some kind of very basic causal
    /// analysis).
    ///
    /// The default implementation always returns `Ok(())`.
    fn check(&self, _m: &HashMap<Self::Name, Self::Value>) -> Result<(), Self::Error> {
        Ok(())
    }
}

/// The `Tautology` struct implements a tautology predicate (i.e. always valid).
pub struct Tautology<N, V> {
    _ph: PhantomData<(N, V)>,
}

impl<N, V> Default for Tautology<N, V> {
    fn default() -> Self {
        Self { _ph: PhantomData }
    }
}

impl<N, V> Predicate for Tautology<N, V> {
    type Name = N;
    type Value = V;
    type Error = ();
}

/// The `LTnVar` struct implements an operator (less than a variable).
pub struct LTnVar<V: Ord, const LHS: char, const RHS: char> {
    _p: PhantomData<V>,
}

impl<V: Ord, const LHS: char, const RHS: char> Default for LTnVar<V, LHS, RHS> {
    fn default() -> Self {
        Self { _p: PhantomData }
    }
}

impl<V: Ord, const LHS: char, const RHS: char> Predicate for LTnVar<V, LHS, RHS>
where
    V: std::cmp::Ord
{
    type Name = char;
    type Value = V;
    type Error = ();

    fn check(&self, m: &HashMap<Self::Name, Self::Value>) -> Result<(), Self::Error> {
        let lhs = m.get(&LHS).ok_or(())?;
        let rhs = m.get(&RHS).ok_or(())?;
        if lhs < rhs {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// The `GTnVar` struct implements an operator (greater than a variable).
pub struct GTnVar<V: Ord, const LHS: char, const RHS: char> {
    _p: PhantomData<V>,
}

impl<V: Ord, const LHS: char, const RHS: char> Default for GTnVar<V, LHS, RHS> {
    fn default() -> Self {
        Self { _p: PhantomData }
    }
}

impl<V: Ord, const LHS: char, const RHS: char> Predicate for GTnVar<V, LHS, RHS>
where
    V: std::cmp::Ord
{
    type Name = char;
    type Value = V;
    type Error = ();

    fn check(&self, m: &HashMap<Self::Name, Self::Value>) -> Result<(), Self::Error> {
        let lhs = m.get(&LHS).ok_or(())?;
        let rhs = m.get(&RHS).ok_or(())?;
        if lhs > rhs {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// The `LTnConst` struct implements an operator (less than a value).
pub struct LTnConst<const LHS: char, const RHS: i32> {}

impl<const LHS: char, const RHS: i32> Default for LTnConst<LHS, RHS> {
    fn default() -> Self { Self {} }
}

impl<const LHS: char, const RHS: i32> Predicate for LTnConst<LHS, RHS>
{
    type Name = char;
    type Value = i32;
    type Error = ();

    fn check(&self, m: &HashMap<Self::Name, Self::Value>) -> Result<(), Self::Error> {
        let lhs = m.get(&LHS).ok_or(())?;
        if lhs < &RHS {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// The `GTnConst` struct implements an operator (less than a value).
pub struct GTnConst<const LHS: char, const RHS: i32> {}

impl<const LHS: char, const RHS: i32> Default for GTnConst<LHS, RHS> {
    fn default() -> Self { Self {} }
}

impl<const LHS: char, const RHS: i32> Predicate for GTnConst<LHS, RHS>
{
    type Name = char;
    type Value = i32;
    type Error = ();

    fn check(&self, m: &HashMap<Self::Name, Self::Value>) -> Result<(), Self::Error> {
        let lhs = m.get(&LHS).ok_or(())?;
        if lhs > &RHS {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// The `EqualVar` struct implements an operator (equal to a variable).
pub struct EqualVar<V: Ord, const LHS: char, const RHS: char> {
    _p: PhantomData<V>,
}

impl<V: Ord, const LHS: char, const RHS: char> Default for EqualVar<V, LHS, RHS> {
    fn default() -> Self {
        Self { _p: PhantomData }
    }
}

impl<V: Ord, const LHS: char, const RHS: char> Predicate for EqualVar<V, LHS, RHS>
where
    V: std::cmp::Ord
{
    type Name = char;
    type Value = V;
    type Error = ();

    fn check(&self, m: &HashMap<Self::Name, Self::Value>) -> Result<(), Self::Error> {
        let lhs = m.get(&LHS).ok_or(())?;
        let rhs = m.get(&RHS).ok_or(())?;
        if lhs == rhs {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// The `EqualConst` struct implements an operator (equal to a value).
pub struct EqualConst<const LHS: char, const RHS: i32> {}

impl<const LHS: char, const RHS: i32> Default for EqualConst<LHS, RHS> {
    fn default() -> Self { Self {} }
}

impl<const LHS: char, const RHS: i32> Predicate for EqualConst<LHS, RHS>
{
    type Name = char;
    type Value = i32;
    type Error = ();

    fn check(&self, m: &HashMap<Self::Name, Self::Value>) -> Result<(), Self::Error> {
        let lhs = m.get(&LHS).ok_or(())?;
        if lhs == &RHS {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// The `LTnThree` struct implements an operator (less than between three operands).
pub struct LTnThree<V: Ord, const LHS: char, const MID: char, const RHS: char> {
    _p: PhantomData<V>,
}

impl<V: Ord, const LHS: char, const MID: char, const RHS: char> Default for LTnThree<V, LHS, MID, RHS> {
    fn default() -> Self {
        Self { _p: PhantomData }
    }
}

impl<V: Ord, const LHS: char, const MID: char, const RHS: char> Predicate for LTnThree<V, LHS, MID, RHS>
where
    V: std::cmp::Ord
{
    type Name = char;
    type Value = V;
    type Error = ();

    fn check(&self, m: &HashMap<Self::Name, Self::Value>) -> Result<(), Self::Error> {
        let lhs = m.get(&LHS).ok_or(())?;
        let mid = m.get(&MID).ok_or(())?;
        let rhs = m.get(&RHS).ok_or(())?;
        if lhs < mid && mid < rhs {
            Ok(())
        } else {
            Err(())
        }
    }
}

/// The `GTnThree` struct implements an operator (greater than between three operands).
pub struct GTnThree<V: Ord, const LHS: char, const MID: char, const RHS: char> {
    _p: PhantomData<V>,
}

impl<V: Ord, const LHS: char, const MID: char, const RHS: char> Default for GTnThree<V, LHS, MID, RHS> {
    fn default() -> Self {
        Self { _p: PhantomData }
    }
}

impl<V: Ord, const LHS: char, const MID: char, const RHS: char> Predicate for GTnThree<V, LHS, MID, RHS>
where
    V: std::cmp::Ord
{
    type Name = char;
    type Value = V;
    type Error = ();

    fn check(&self, m: &HashMap<Self::Name, Self::Value>) -> Result<(), Self::Error> {
        let lhs = m.get(&LHS).ok_or(())?;
        let mid = m.get(&MID).ok_or(())?;
        let rhs = m.get(&RHS).ok_or(())?;
        if lhs > mid && mid > rhs {
            Ok(())
        } else {
            Err(())
        }
    }
}

pub struct LTn<N, V> {
    lhs: N,
    rhs: N,
    _p: PhantomData<V>,
}

impl<N: Eq + Hash, V: PartialOrd> LTn<N, V> {
    fn new(lhs: N, rhs: N) -> Self {
        Self{ lhs, rhs, _p: PhantomData }
    }

    fn check(&self, m: &HashMap<N, V>) -> Result<(), ()> {
        let lhs = m.get(&self.lhs).ok_or(())?;
        let rhs = m.get(&self.rhs).ok_or(())?;
        if lhs < rhs {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[macro_export]
macro_rules! formula {
    ( $lhs:ident < $rhs:ident ) => {
        LTn::new($lhs, $rhs)
    }
}

pub enum Formula<N, V> {
    LTn(LTn<N, V>)
}

#[derive(Parser)]
#[grammar = "parser/predicate.pest"]
struct PredicateParser;

impl<'a, N, V> From<Pair<'a, Rule>> for LTn<N, V> 
where N: From<&'a str> + std::hash::Hash + std::cmp::Eq,
      V: std::cmp::Ord
{
    fn from(p: Pair<'a, Rule>) -> LTn<N, V> 
    {
        let mut pairs = p.into_inner();
        let lhs: N = pairs.next().unwrap().as_str().into();
        let rhs: N = pairs.next().unwrap().as_str().into();

        LTn::new(lhs, rhs)
    }
}

impl<'a, N, V> TryFrom<&'a str> for Formula<N, V>
where N: From<&'a str> + std::hash::Hash + std::cmp::Eq,
      V: std::cmp::Ord
{
    type Error = Error<Rule>;

    fn try_from(s: &'a str) -> Result<Formula<N, V>, Error<Rule>> {
        let mut pairs = PredicateParser::parse(Rule::ltn , s)?;
        let pair = pairs.next().unwrap();
        let formula = match pair.as_rule() {
            Rule::ltn => Formula::LTn(pair.into()),
            _ => panic!(),
        };
        Ok(formula)
    }
}
