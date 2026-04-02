#[macro_use]
#[allow(unused_macros)]
mod generated_contracts;
pub mod verification_specs;

pub fn add(left: u64, right: u64) -> u64 {
    contract_pre_configuration!();
    let result = left + right;
    contract_post_configuration!(&"ok");
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }
}
