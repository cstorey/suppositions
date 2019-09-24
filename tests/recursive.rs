extern crate env_logger;
#[macro_use]
extern crate log;
extern crate suppositions;

use std::ops::Add;
use suppositions::generators::*;
use suppositions::*;

// Hutton's razor as an example.
// Demonstrates recursive generators.

#[derive(Debug, Clone)]
pub enum Expr {
    Lit(u8),
    Add(Box<Expr>, Box<Expr>),
}

impl Add<Expr> for Expr {
    type Output = Expr;
    fn add(self, other: Expr) -> Expr {
        Expr::Add(Box::new(self), Box::new(other))
    }
}

impl Expr {
    fn eval(&self) -> u64 {
        match self {
            &Expr::Lit(n) => n as u64,
            &Expr::Add(ref a, ref b) => a.eval() + b.eval(),
        }
    }
}

fn expr_gen() -> Box<dyn GeneratorObject<Item = Expr>> {
    let lit = u8s().map(Expr::Lit);
    let lit2 = u8s().map(Expr::Lit);
    let add = (lazy(expr_gen), lazy(expr_gen)).map(|(a, b)| Expr::Add(Box::new(a), Box::new(b)));

    // In lieu of having weighted choice
    one_of(lit).or(lit2).or(add).boxed()
}

#[test]
fn add_adds() {
    env_logger::try_init().unwrap_or_default();
    property((expr_gen(), expr_gen())).check(|(a, b)| {
        debug!("Testing: {:?} + {:?}", a, b);
        assert_eq!(a.eval() + b.eval(), (a + b).eval())
    });
}
