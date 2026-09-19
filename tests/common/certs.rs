use rcgen::{
    BasicConstraints, Certificate, CertificateParams, DistinguishedName, DnType, Error, IsCa,
    Issuer, KeyPair, KeyUsagePurpose, PKCS_ECDSA_P256_SHA256,
};

pub const LOCALHOST: &str = "127.0.0.1";

pub struct Certs {
    pub ca_cert: Certificate,
    pub server_cert: Certificate,
    pub server_key: KeyPair,
    pub client_cert: Certificate,
    pub client_key: KeyPair,
}

pub fn gen_certs() -> Result<Certs, Error> {
    let ca_key = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256)?;
    let server_key = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256)?;
    let client_key = KeyPair::generate_for(&PKCS_ECDSA_P256_SHA256)?;

    let mut ca_dn = DistinguishedName::new();
    ca_dn.push(DnType::CommonName, "My Root CA");

    let mut dn = DistinguishedName::new();
    dn.push(DnType::CommonName, "My Service");

    let mut ca_params = CertificateParams::default();
    ca_params.distinguished_name = ca_dn;
    ca_params.is_ca = IsCa::Ca(BasicConstraints::Unconstrained);
    ca_params.key_usages = vec![KeyUsagePurpose::KeyCertSign, KeyUsagePurpose::CrlSign];

    let ca_cert = ca_params.self_signed(&ca_key)?;

    let mut server_params = CertificateParams::new(vec![LOCALHOST.to_string()])?;
    server_params.distinguished_name = dn.clone();

    let mut client_params = CertificateParams::new(vec!["client.com".to_string()])?;
    client_params.distinguished_name = dn.clone();

    let issuer = Issuer::from_params(&ca_params, ca_key);
    let server_cert = server_params.signed_by(&server_key, &issuer)?;
    let client_cert = client_params.signed_by(&client_key, &issuer)?;

    Ok(Certs {
        ca_cert,
        server_cert,
        server_key,
        client_cert,
        client_key,
    })
}
