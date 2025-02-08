use rand::Rng;

const ID_LEN: usize = 7;

pub fn generate() -> String {
    rand::rng()
        .sample_iter(&rand::distr::Alphanumeric)
        .take(ID_LEN)
        .map(char::from)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_id() {
        let link_id = generate();

        assert_eq!(link_id.len(), ID_LEN);
    }
}
