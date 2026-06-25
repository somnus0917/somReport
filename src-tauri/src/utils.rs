pub fn estimate_cost_cents(
    input_tokens: i64,
    output_tokens: i64,
    input_cost_per_million_cents: f64,
    output_cost_per_million_cents: f64,
) -> f64 {
    (input_tokens as f64 * input_cost_per_million_cents
        + output_tokens as f64 * output_cost_per_million_cents)
        / 1_000_000.0
}
