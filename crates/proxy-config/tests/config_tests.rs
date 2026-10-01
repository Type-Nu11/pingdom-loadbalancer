use proxy_config::load_and_validate;

#[test]
fn edge_config_is_valid() {
    let config = load_and_validate("../../configs/edge.toml").expect("edge config must be valid");

    assert_eq!(config.proxy.name, "typenull-edge");
    assert_eq!(config.tls.hostname, "www.typenull.xyz");
    assert_eq!(config.routing.target_header, "X-Client-Type");
}
