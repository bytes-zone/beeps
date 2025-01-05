#[tracing::instrument]
pub async fn handler() -> &'static str {
    "OK"
}

#[cfg(test)]
mod test {
    use super::*;

    #[test_log::test(tokio::test)]
    async fn test_success() {
        assert_eq!(handler().await, "OK")
    }
}
