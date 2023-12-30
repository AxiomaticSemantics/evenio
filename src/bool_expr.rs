use core::{fmt, mem};

use crate::bit_set::BitSet;
use crate::sparse::SparseIndex;

pub struct BoolExpr<T> {
    // The boolean expression in disjunctive normal form,
    // e.g. (A ∧ B ∧ ¬C) ∨ (D ∧ ¬E ∧ ¬F). This is an "OR of ANDs".
    ands: Vec<Ands<T>>,
}

struct Ands<T> {
    vars: BitSet<T>,
    negated_vars: BitSet<T>,
}

impl<T> Ands<T> {
    fn new() -> Self {
        Self {
            vars: BitSet::new(),
            negated_vars: BitSet::new(),
        }
    }
}

impl<T> BoolExpr<T> {
    /// Returns an access expression representing `false` or `0`. This is the
    /// identity element for `∨`.
    pub fn zero() -> Self {
        Self { ands: vec![] }
    }

    /// Returns a new expression representing `true` or `1`. This is the
    /// identity element for `∧`.
    pub fn one() -> Self {
        Self {
            ands: vec![Ands::new()],
        }
    }

    pub fn with(value: T) -> Self
    where
        T: SparseIndex,
    {
        Self {
            ands: vec![Ands {
                vars: {
                    let mut vars = BitSet::new();
                    vars.insert(value);
                    vars
                },
                negated_vars: BitSet::new(),
            }],
        }
    }

    pub fn without(value: T) -> Self
    where
        T: SparseIndex,
    {
        Self {
            ands: vec![Ands {
                vars: BitSet::new(),
                negated_vars: {
                    let mut negated_vars = BitSet::new();
                    negated_vars.insert(value);
                    negated_vars
                },
            }],
        }
    }

    /// Evaluate the boolean expression. `get_var` provides the values of the
    /// variables in the expression.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use evenio::bool_expr::BoolExpr;
    ///
    /// const A: u32 = 0;
    /// const B: u32 = 1;
    ///
    /// let expr = BoolExpr::with(A).xor(&BoolExpr::with(B));
    ///
    /// let get_var = |a, b| {
    ///     move |var| match var {
    ///         A => a,
    ///         B => b,
    ///         _ => false,
    ///     }
    /// };
    ///
    /// assert_eq!(expr.eval(get_var(false, false)), false);
    /// assert_eq!(expr.eval(get_var(true, false)), true);
    /// assert_eq!(expr.eval(get_var(false, true)), true);
    /// assert_eq!(expr.eval(get_var(true, true)), false);
    /// ```
    pub fn eval<F>(&self, mut get_var: F) -> bool
    where
        T: SparseIndex,
        F: FnMut(T) -> bool,
    {
        'ands: for ands in &self.ands {
            for var in ands.vars.iter() {
                if !get_var(var) {
                    continue 'ands;
                }
            }

            for var in ands.negated_vars.iter() {
                if get_var(var) {
                    continue 'ands;
                }
            }

            return true;
        }

        false
    }

    #[must_use]
    pub fn and(mut self, other: &Self) -> Self
    where
        T: SparseIndex,
    {
        let mut res = Vec::new();
        for this in &self.ands {
            for other in &other.ands {
                let mut new_ands = this.clone();

                new_ands.vars |= &other.vars;
                new_ands.negated_vars |= &other.negated_vars;

                // Skip contradictions.
                if new_ands.vars.is_disjoint(&new_ands.negated_vars) {
                    res.push(new_ands);
                }
            }
        }

        self.ands = res;
        self
    }

    #[must_use]
    pub fn or(mut self, other: &Self) -> Self
    where
        T: SparseIndex,
    {
        self.ands.extend(other.ands.iter().cloned());
        self
    }

    #[must_use]
    pub fn not(mut self) -> Self
    where
        T: SparseIndex,
    {
        let mut res = Self::one();

        // Apply De Morgan's laws.
        for mut ands in mem::take(&mut self.ands) {
            let mut ors = Self::zero();

            mem::swap(&mut ands.vars, &mut ands.negated_vars);

            for var in ands.vars.iter() {
                let mut a = Ands::new();
                a.vars.insert(var);
                ors.ands.push(a);
            }

            for negated_var in ands.negated_vars.iter() {
                let mut a = Ands::new();
                a.negated_vars.insert(negated_var);
                ors.ands.push(a);
            }

            res = res.and(&ors);
        }

        res
    }

    pub fn xor(self, other: &Self) -> Self
    where
        T: SparseIndex,
    {
        // A ⊻ B ≡ (A ∧ ¬B) ∨ (B ∧ ¬A)
        self.clone()
            .and(&other.clone().not())
            .or(&other.clone().and(&self.not()))
    }

    /// Determines if `self` and `other` are disjoint, i.e. if there is no
    /// combination of values the variables could have to make both expressions
    /// true at the same time.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use evenio::bool_expr::BoolExpr;
    ///
    /// // `A` is not disjoint with `B`
    /// assert!(!BoolExpr::with(A).is_disjoint(&BoolExpr::with(B)));
    ///
    /// // `A` is disjoint with `¬A`.
    /// assert!(BoolExpr::with(A).is_disjoint(&BoolExpr::without(A)));
    ///
    /// // `A ∧ ¬A` is disjoint with `B ∧ C`.
    /// let left = BoolExpr::with(A).and(&BoolExpr::without(A));
    /// let right = BoolExpr::with(C).and(&BoolExpr::with(D));
    /// assert!(left.is_disjoint(&right));
    ///
    /// const A: u32 = 0;
    /// const B: u32 = 1;
    /// const C: u32 = 2;
    /// const D: u32 = 3;
    /// ```
    pub fn is_disjoint(&self, other: &Self) -> bool {
        self.ands.iter().all(|this| {
            other.ands.iter().all(|other| {
                !this.vars.is_disjoint(&this.negated_vars)
                    || !other.vars.is_disjoint(&other.negated_vars)
                    || !this.vars.is_disjoint(&other.negated_vars)
                    || !other.vars.is_disjoint(&this.negated_vars)
            })
        })
    }
}

impl<T> Clone for BoolExpr<T> {
    fn clone(&self) -> Self {
        Self {
            ands: self.ands.clone(),
        }
    }
}

impl<T> Clone for Ands<T> {
    fn clone(&self) -> Self {
        Self {
            vars: self.vars.clone(),
            negated_vars: self.negated_vars.clone(),
        }
    }
}

impl<T> fmt::Debug for BoolExpr<T>
where
    T: SparseIndex + fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.ands.is_empty() {
            write!(f, "⊥")?;
        } else {
            let mut first = true;

            for ands in self.ands.iter() {
                if !first {
                    write!(f, " ∨ ")?;
                }
                first = false;

                if ands.vars.is_empty() && ands.negated_vars.is_empty() {
                    write!(f, "⊤")?;
                } else {
                    let mut first = true;

                    for var in ands.vars.iter() {
                        if !first {
                            write!(f, " ∧ ")?;
                        }
                        first = false;

                        write!(f, "{var:?}")?;
                    }

                    for var in ands.negated_vars.iter() {
                        if !first {
                            write!(f, " ∧ ")?;
                        }
                        first = false;

                        write!(f, "¬{var:?}")?;
                    }
                }
            }
        }

        Ok(())
    }
}