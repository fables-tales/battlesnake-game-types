pub mod compact_representation;
mod cross_product;
/// Types for working with [battlesnake](https://docs.battlesnake.com/).
/// The goal is to provide simulation tooling and fast representations that
/// enable development of efficient minmax/MCTS
pub mod types;
pub mod wire_representation;
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
