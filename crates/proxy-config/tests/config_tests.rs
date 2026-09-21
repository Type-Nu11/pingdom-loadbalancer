use proxy_config::load_and_validate;

#[test]
fn edge_config_is_valid() {
    let config = load_and_validate("../../configs/edge.toml").expect("edge config must be valid");

    assert_eq!(config.proxy.name, "typenull-edge");
    assert_eq!(config.routes.len(), 3);
    assert_eq!(config.backends.len(), 3);
}
