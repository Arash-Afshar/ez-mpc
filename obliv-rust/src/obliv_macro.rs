//! This module implements the macros that will expand the `obliv` keyword into an MPC protocol.
//!

#[macro_export]
macro_rules! obliv {
    ($a:ident + $b:ident) => {{
        {
            let val: usize = $a + $b;
            val
        }
    }};
    ($a:ident - $b:ident) => {{
        {
            let val: usize = $a - $b;
            val
        }
    }};
}

#[macro_export]
macro_rules! obliv_assign {
    ($a:ident <- party $b:expr, value $c:expr) => {{
        {
            $c
        }
    }};
}

fn setup() {
    // gates
    // gates.update(a, b)
}

#[cfg(test)]
mod tests {

    #[test]
    fn obliv_add() {
        let a = 1;
        let b = 2;
        let val = obliv!(a + b);
        assert_eq!(val, 3);
    }

    #[test]
    fn obliv_sub() {
        let a = 4;
        let b = 3;
        let val = obliv!(a - b);
        assert_eq!(val, 1);
    }

    #[test]
    fn obliv_init() {
        let val = obliv_assign!(a <- party 1, value 3*4);
        assert_eq!(val, 12);
    }
}
