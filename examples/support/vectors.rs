pub fn values(id: usize, dimensions: usize) -> Vec<f32> {
    (0..dimensions)
        .map(|d| {
            (((id as u64)
                .wrapping_mul(6364136223846793005)
                .wrapping_add((d as u64).wrapping_mul(1442695040888963407).wrapping_add(7)))
                % 10007) as f32
                - 5003.0
        })
        .collect()
}
