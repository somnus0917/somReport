pub fn estimate_cost_yuan(
    input_tokens: i64,
    output_tokens: i64,
    input_cost_per_million_yuan: f64,
    output_cost_per_million_yuan: f64,
) -> f64 {
    (input_tokens as f64 * input_cost_per_million_yuan
        + output_tokens as f64 * output_cost_per_million_yuan)
        / 1_000_000.0
}
