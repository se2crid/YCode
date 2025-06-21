use hex;
use icloud_auth::{AppleAccount, DeveloperDeviceType, DeveloperTeam};
use openssl::{
    hash::MessageDigest,
    pkey::{PKey, Private},
    rsa::Rsa,
    x509::{X509Name, X509ReqBuilder, X509},
};
use sha1::{Digest, Sha1};
use std::{fs, path::PathBuf, sync::Arc};

#[derive(Debug, Clone)]
pub struct CertificateIdentity {
    pub certificate: Option<X509>,
    pub private_key: PKey<Private>,
    pub key_file: PathBuf,
}

impl CertificateIdentity {
    pub async fn new(
        configuration_path: PathBuf,
        apple_account: &Arc<AppleAccount>,
        apple_id: String,
    ) -> Result<Self, String> {
        let mut hasher = Sha1::new();
        hasher.update(apple_id.as_bytes());
        let hash_string = hex::encode(hasher.finalize()).to_lowercase();
        let key_path = configuration_path.join("keys").join(hash_string);
        fs::create_dir_all(&key_path)
            .map_err(|e| format!("Failed to create key directory: {}", e))?;

        let key_file = key_path.join("key.pem");
        let teams = apple_account
            .list_teams()
            .await
            .map_err(|e| format!("Failed to list teams: {:?}", e))?;
        let team = teams.first().ok_or("No teams found")?;
        let private_key = if key_file.exists() {
            let key_data = fs::read_to_string(&key_file)
                .map_err(|e| format!("Failed to read key file: {}", e))?;
            PKey::private_key_from_pem(key_data.as_bytes())
                .map_err(|e| format!("Failed to load private key: {}", e))?
        } else {
            let rsa =
                Rsa::generate(2048).map_err(|e| format!("Failed to generate RSA key: {}", e))?;
            let key =
                PKey::from_rsa(rsa).map_err(|e| format!("Failed to create private key: {}", e))?;
            let pem_data = key
                .private_key_to_pem_pkcs8()
                .map_err(|e| format!("Failed to encode private key: {}", e))?;
            fs::write(&key_file, pem_data)
                .map_err(|e| format!("Failed to save key file: {}", e))?;
            key
        };

        let mut cert_identity = CertificateIdentity {
            certificate: None,
            private_key,
            key_file,
        };

        if let Ok(cert) = cert_identity
            .find_matching_certificate(&apple_account, team)
            .await
        {
            cert_identity.certificate = Some(cert);
            return Ok(cert_identity);
        }

        cert_identity
            .request_new_certificate(&apple_account, team)
            .await?;
        Ok(cert_identity)
    }

    async fn find_matching_certificate(
        &self,
        apple_account: &AppleAccount,
        team: &DeveloperTeam,
    ) -> Result<X509, String> {
        let certificates = apple_account
            .list_all_development_certs(DeveloperDeviceType::Ios, team)
            .await
            .map_err(|e| format!("Failed to list certificates: {:?}", e))?;

        let our_public_key = self
            .private_key
            .public_key_to_der()
            .map_err(|e| format!("Failed to get public key: {}", e))?;

        for cert in certificates
            .iter()
            .filter(|c| c.machine_name == "YCode".to_string())
        {
            if let Ok(x509_cert) = X509::from_der(&cert.cert_content) {
                if let Ok(cert_public_key) = x509_cert.public_key() {
                    if let Ok(cert_public_key_der) = cert_public_key.public_key_to_der() {
                        if cert_public_key_der == our_public_key {
                            return Ok(x509_cert);
                        }
                    }
                }
            }
        }
        Err("No matching certificate found".to_string())
    }

    async fn request_new_certificate(
        &mut self,
        apple_account: &AppleAccount,
        team: &DeveloperTeam,
    ) -> Result<(), String> {
        let mut req_builder = X509ReqBuilder::new()
            .map_err(|e| format!("Failed to create request builder: {}", e))?;
        let mut name_builder =
            X509Name::builder().map_err(|e| format!("Failed to create name builder: {}", e))?;

        name_builder
            .append_entry_by_text("C", "US")
            .map_err(|e| format!("Failed to set country: {}", e))?;
        name_builder
            .append_entry_by_text("ST", "STATE")
            .map_err(|e| format!("Failed to set state: {}", e))?;
        name_builder
            .append_entry_by_text("L", "LOCAL")
            .map_err(|e| format!("Failed to set locality: {}", e))?;
        name_builder
            .append_entry_by_text("O", "ORGNIZATION")
            .map_err(|e| format!("Failed to set organization: {}", e))?;
        name_builder
            .append_entry_by_text("CN", "CN")
            .map_err(|e| format!("Failed to set common name: {}", e))?;

        req_builder
            .set_subject_name(&name_builder.build())
            .map_err(|e| format!("Failed to set subject name: {}", e))?;
        req_builder
            .set_pubkey(&self.private_key)
            .map_err(|e| format!("Failed to set public key: {}", e))?;
        req_builder
            .sign(&self.private_key, MessageDigest::sha256())
            .map_err(|e| format!("Failed to sign request: {}", e))?;

        let csr_pem = req_builder
            .build()
            .to_pem()
            .map_err(|e| format!("Failed to encode CSR: {}", e))?;

        let certificate_id = apple_account
            .submit_development_csr(
                DeveloperDeviceType::Ios,
                team,
                String::from_utf8_lossy(&csr_pem).to_string(),
            )
            .await
            .map_err(|e| format!("Failed to submit CSR: {:?}", e))?;

        let certificates = apple_account
            .list_all_development_certs(DeveloperDeviceType::Ios, team)
            .await
            .map_err(|e| format!("Failed to list certificates: {:?}", e))?;

        let apple_cert = certificates
            .iter()
            .find(|cert| cert.certificate_id == certificate_id)
            .ok_or("Certificate not found after submission")?;

        self.certificate = Some(
            X509::from_der(&apple_cert.cert_content)
                .map_err(|e| format!("Failed to parse certificate: {}", e))?,
        );

        Ok(())
    }

    pub fn get_certificate(&self) -> Option<&X509> {
        self.certificate.as_ref()
    }

    pub fn get_private_key(&self) -> &PKey<Private> {
        &self.private_key
    }
}
