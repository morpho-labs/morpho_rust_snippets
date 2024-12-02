use alloy::primitives::{utils::parse_units, U256};

pub fn mul_div_down(x: U256, y: U256, z: U256) -> U256 {
    (x * y) / z
}

pub fn w_mul_down(x: U256, y: U256) -> U256 {
    mul_div_down(x, y, parse_units("1.0", "ether").unwrap().into())
}

pub fn w_taylor_compounded(x: U256, n: U256) -> U256 {
    let first_term = x * n;
    let second_term = mul_div_down(
        first_term,
        first_term,
        parse_units("2.0", "ether").unwrap().into(),
    );
    let third_term = mul_div_down(
        second_term,
        first_term,
        parse_units("3.0", "ether").unwrap().into(),
    );

    first_term + second_term + third_term
}
