pub fn clip<T>(x: T, lb: T, ub: T) -> T
where
    T: core::cmp::Ord,
{
    use core::cmp::{max, min};
    max(min(x, ub), lb)
}
