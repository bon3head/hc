# Graph Report - sigstore-rs  (2026-11-25)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 1968 nodes · 4300 edges · 88 communities (75 shown, 13 thin omitted)
- Extraction: 94% EXTRACTED · 6% INFERRED · 0% AMBIGUOUS · INFERRED: 265 edges (avg confidence: 0.68)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Community 0
- Community 1
- Community 2
- Community 3
- Community 4
- Community 5
- Community 6
- Community 7
- Community 8
- Community 9
- Community 10
- Community 11
- Community 12
- Community 13
- Community 14
- Community 15
- Community 16
- Community 17
- Community 18
- Community 19
- Community 20
- Community 21
- Community 22
- Community 23
- Community 24
- Community 25
- Community 26
- Community 27
- Community 28
- Community 29
- Community 31
- Community 32
- Community 33
- Community 34
- Community 35
- Community 36
- Community 37
- Community 38
- Community 39
- Community 40
- Community 41
- Community 42
- Community 43
- Community 44
- Community 45
- Community 46
- Community 47
- Community 48
- Community 49
- Community 50
- Community 51
- Community 52
- Community 53
- Community 54
- Community 55
- Community 56
- Community 57
- Community 59
- Community 60
- Community 61
- Community 62
- Community 63
- Community 64
- Community 65
- Community 66
- Community 67
- Community 68
- Community 69
- Community 70
- Community 71
- Community 72
- Community 73
- Community 74
- Community 75
- Community 76
- Community 77
- Community 78
- Community 80
- Community 81
- Community 82
- Community 83
- Community 84
- Community 85
- Community 86

## God Nodes (most connected - your core abstractions)
1. `SigstoreError` - 122 edges
2. `CosignVerificationKey` - 67 edges
3. `Value` - 62 edges
4. `SigningScheme` - 46 edges
5. `SignatureLayer` - 44 edges
6. `Certificate` - 37 edges
7. `OciReference` - 31 edges
8. `SigStoreSigner` - 30 edges
9. `KeyPair` - 29 edges
10. `Client` - 29 edges

## Surprising Connections (you probably didn't know these)
- `SignatureErrorKind` --references--> `SigstoreError`  [INFERRED]
  src/bundle/verify/models.rs → src/errors.rs
- `material_from_private()` --references--> `SigstoreError`  [INFERRED]
  src/crypto/backend/openssl.rs → src/errors.rs
- `SigstoreError` --references--> `KeyringError`  [INFERRED]
  src/errors.rs → src/crypto/keyring.rs
- `SigstoreError` --references--> `SCTError`  [INFERRED]
  src/errors.rs → src/crypto/transparency.rs
- `SigstoreError` --references--> `DecodeError`  [INFERRED]
  src/errors.rs → src/rekor/models/checkpoint.rs

## Import Cycles
- 1-file cycle: `src/bundle/sign.rs -> src/bundle/sign.rs`
- 1-file cycle: `src/cosign/mod.rs -> src/cosign/mod.rs`
- 1-file cycle: `src/rekor/models/checkpoint.rs -> src/rekor/models/checkpoint.rs`
- 1-file cycle: `src/rekor/models/log_entry.rs -> src/rekor/models/log_entry.rs`
- 1-file cycle: `src/rekor/models/log_info.rs -> src/rekor/models/log_info.rs`
- 1-file cycle: `src/bundle/verify/verifier.rs -> src/bundle/verify/verifier.rs`
- 1-file cycle: `src/crypto/signing_key/ecdsa/ec.rs -> src/crypto/signing_key/ecdsa/ec.rs`
- 1-file cycle: `src/crypto/signing_key/ed25519.rs -> src/crypto/signing_key/ed25519.rs`
- 1-file cycle: `src/crypto/signing_key/rsa/keypair.rs -> src/crypto/signing_key/rsa/keypair.rs`
- 1-file cycle: `src/crypto/signing_key/rsa/mod.rs -> src/crypto/signing_key/rsa/mod.rs`
- 1-file cycle: `src/cosign/verification_constraint/cert_subject_email_verifier.rs -> src/cosign/verification_constraint/cert_subject_email_verifier.rs`
- 1-file cycle: `src/crypto/backend/mod.rs -> src/crypto/backend/mod.rs`
- 1-file cycle: `src/crypto/backend/openssl.rs -> src/crypto/backend/openssl.rs`
- 1-file cycle: `src/bundle/verify/models.rs -> src/bundle/verify/models.rs`
- 1-file cycle: `src/crypto/transparency.rs -> src/crypto/transparency.rs`
- 1-file cycle: `src/crypto/merkle/proof_verification.rs -> src/crypto/merkle/proof_verification.rs`
- 1-file cycle: `src/crypto/keyring.rs -> src/crypto/keyring.rs`
- 1-file cycle: `src/crypto/merkle/rfc6962.rs -> src/crypto/merkle/rfc6962.rs`
- 2-file cycle: `src/cosign/client.rs -> src/cosign/mod.rs -> src/cosign/client.rs`
- 2-file cycle: `src/cosign/mod.rs -> src/cosign/signature_layers.rs -> src/cosign/mod.rs`

## Communities (88 total, 13 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.03
Nodes (72): SigstoreError, ApplyConstraintError, Base64DecodeError, CertificateExpiredBeforeSignaturesSubmittedToRekor, CertificateInvalidEmail, CertificateIssuedAfterSignaturesSubmittedToRekor, CertificateParsingError, CertificatePoolError (+64 more)

### Community 1 - "Community 1"
Cohesion: 0.06
Nodes (50): Key, KeyMaterial, P256, Result, verify_keyring(), generate_ecdsa_p256_key_pair(), generate_ecdsa_p384_key_pair(), generate_ed25519_key_pair() (+42 more)

### Community 2 - "Community 2"
Cohesion: 0.05
Nodes (61): TransparencyLogEntry, artifact_digest_from_hasher(), ArtifactDigest, Sha256, Sha512, Bundle01ProfileErrorKind, InclusionPromiseMissing, Bundle02ProfileErrorKind (+53 more)

### Community 3 - "Community 3"
Cohesion: 0.06
Nodes (47): DigestWriter, SigningArtifact, SigningSession<'ctx>, NPM_CERT_IDENTITY, NPM_CERT_ISSUER, NPM_SLSA_BUNDLE_V03, NPM_TARBALL_SHA512, SIGSTORE_TRUSTED_ROOT (+39 more)

### Community 4 - "Community 4"
Cohesion: 0.06
Nodes (29): ecdsa_generate(), ecdsa_spki_from_pkcs8(), backend_curve(), curve_from_pkcs8_der(), curve_from_pkcs8_der_detects_curve(), curve_from_pkcs8_der_invalid_der(), curve_from_pkcs8_der_unsupported_oid(), ecdsa_derive_verification_key() (+21 more)

### Community 5 - "Community 5"
Cohesion: 0.06
Nodes (28): AlpineAllOf, Alpine, Value, HashedrekordAllOf, HelmAllOf, Helm, IntotoAllOf, Intoto (+20 more)

### Community 6 - "Community 6"
Cohesion: 0.07
Nodes (12): DEFAULT_CLIENT_ID, DEFAULT_CLIENT_SECRET, DEFAULT_ISSUER, DEFAULT_REDIRECT_PORT, AsyncReqwestClient, SyncReqwestClient, assert_auth_url(), OpenIDAuthorize (+4 more)

### Community 7 - "Community 7"
Cohesion: 0.06
Nodes (28): fill_random(), BOX_KEY_SIZE, BOX_NONCE_SIZE, Data, decrypt(), encrypt(), from_base64(), generate_random() (+20 more)

### Community 8 - "Community 8"
Cohesion: 0.07
Nodes (18): acceptable_rsa_public_exponent(), CERT_ALGS, cert_digest(), curve_digest(), ecdsa_generate(), EcdsaSigningKey, ed25519_generate(), ed25519_pkey() (+10 more)

### Community 9 - "Community 9"
Cohesion: 0.06
Nodes (13): ecdsa_curve_from_pkcs8(), ecdsa_generate(), ecdsa_signing_alg(), ecdsa_spki_from_pkcs8(), EcdsaSigningKey, ed25519_generate(), ed25519_sign(), ed25519_spki_from_pkcs8() (+5 more)

### Community 10 - "Community 10"
Cohesion: 0.11
Nodes (23): build_correct_signature_layer_with_certificate(), build_correct_signature_layer_without_bundle(), is_signed_by_key(), is_signed_by_key_fails_when_signature_is_not_valid(), SignatureLayer, AnnotationVerifier, cert_email_verifier_email_and_issuer(), cert_email_verifier_email_and_issuer_regex() (+15 more)

### Community 11 - "Community 11"
Cohesion: 0.10
Nodes (15): Checkpoint, CheckpointSignature, OtherContent, KeyValue, ParseCheckpointError, DecodeError, SignedCheckpoint, test_checkpoint_encode_decode_multiple_signatures() (+7 more)

### Community 12 - "Community 12"
Cohesion: 0.10
Nodes (15): ed25519_sign(), ed25519_derive_verification_key(), ed25519_error_unencrypted_pem_password(), ed25519_from_encrypted_pem(), ed25519_from_unencrypted_pem(), ed25519_generate_public_key(), ed25519_sign_and_verify(), ed25519_to_and_from_der() (+7 more)

### Community 13 - "Community 13"
Cohesion: 0.10
Nodes (29): build_bundle(), bundle_with_dsse_sig_count(), bundle_with_tlog_entry_count(), BundleConsistencyInputs, certificate_signature_from_certificate_using_email(), certificate_signature_from_certificate_using_uri(), certificate_signature_from_certificate_without_email_and_uri(), from_sigstore_bundle() (+21 more)

### Community 14 - "Community 14"
Cohesion: 0.08
Nodes (10): BASE64_IGNORED_NEWLINES, CertData, CertGenerationOptions, chrono_to_offset_date_time(), PUBLIC_KEY, Signature, Base64Encoded, signature_decode_bytes() (+2 more)

### Community 15 - "Community 15"
Cohesion: 0.10
Nodes (28): ApiKey, BasicAuth, Configuration, VERSION, create_log_entry(), CreateLogEntryError, DefaultResponse, Status400 (+20 more)

### Community 16 - "Community 16"
Cohesion: 0.14
Nodes (9): fetch_manifest_digest_cached(), OciCachingClient, pull_cached(), pull_manifest_cached(), pull_referrers_cached(), PullManifestSettings, PullReferrersSettings, PullSettings (+1 more)

### Community 17 - "Community 17"
Cohesion: 0.10
Nodes (16): ecdsa_scheme(), ecdsa_sign(), ecdsa_signing_key_reused(), EcdsaCurve, P256, P384, P521, message_digest() (+8 more)

### Community 18 - "Community 18"
Cohesion: 0.13
Nodes (17): rsa_from_pkcs1(), rsa_generate(), EMPTY_PASSWORD, KEY_SIZE, modulus_to_key_size(), PASSWORD, rsa_derive_verification_key(), rsa_error_unencrypted_pem_password() (+9 more)

### Community 19 - "Community 19"
Cohesion: 0.10
Nodes (13): COSIGN_PRIVATE_KEY_PEM_LABEL, sigstore_keypair_code!, sigstore_keypair_from!, PRIVATE_KEY_PEM_LABEL, PUBLIC_KEY_PEM_LABEL, RSA_PRIVATE_KEY_PEM_LABEL, SIGSTORE_PRIVATE_KEY_PEM_LABEL, sigstore_signing() (+5 more)

### Community 20 - "Community 20"
Cohesion: 0.10
Nodes (12): Body, CONTENT_TYPE_HEADER_NAME, Csr, FULCIO_ROOT, FulcioCert, FulcioClient, PublicKey, SIGNING_CERT_PATH (+4 more)

### Community 21 - "Community 21"
Cohesion: 0.11
Nodes (12): message_signature_rejects_non_sha256_digest(), AnnotationMarker, Constraint, SignConstraintVec, PrivateKeySigner, SigningScheme, ECDSA_P256_SHA256_ASN1, ECDSA_P384_SHA384_ASN1 (+4 more)

### Community 22 - "Community 22"
Cohesion: 0.11
Nodes (18): accepts_buffer_at_the_minimum_length(), CertificateChain, CertificateResponse, decode(), deserialize_inner_detached_sct(), deserialize_inner_detached_sct_signature(), InnerDetachedSCT, rejects_buffer_shorter_than_the_length_prefix() (+10 more)

### Community 23 - "Community 23"
Cohesion: 0.12
Nodes (12): build_test_client(), CONFIG_DATA, fetch_from_referrers_propagates_registry_error(), fetch_from_referrers_returns_empty(), fetch_from_tag_returns_err(), index_with_one_entry(), triangulate_sigstore_object(), Auth (+4 more)

### Community 24 - "Community 24"
Cohesion: 0.09
Nodes (8): EMBEDDED_SCT_LEAF_PEM, pem_to_der(), PreCert, precert_tbs_der(), precert_tbs_der_keeps_the_original_encoding(), verify_detached_sct(), verify_embedded_sct(), verify_sct()

### Community 25 - "Community 25"
Cohesion: 0.13
Nodes (8): DigestWriter<'_, D>, Client, BytesStreamAdapter, file_found_on_disk(), file_not_found_on_disk(), ReqwestTransport, transport(), unsupported_scheme()

### Community 26 - "Community 26"
Cohesion: 0.11
Nodes (9): AllOf, AnyOf, Identity, impl_policy!, oids!, PolicyResult, SingleX509ExtPolicy, T (+1 more)

### Community 27 - "Community 27"
Cohesion: 0.14
Nodes (13): TreeSize, ConsistencyProof, RekorConsistencyProof, InactiveShardLogInfo, InclusionProof, LOG_INFO_NEW, LOG_INFO_OLD, LOG_PROOF (+5 more)

### Community 28 - "Community 28"
Cohesion: 0.13
Nodes (18): add_constrain_some_failed(), add_constrains_all_succeed(), apply_constraints(), build_remote_cosign_client(), FULCIO_CRT_1_PEM, FULCIO_CRT_2_PEM, GITHUB_ACTIONS_OIDC_ISSUER, KUBEWARDEN_CONTROLLER_IMAGE (+10 more)

### Community 29 - "Community 29"
Cohesion: 0.17
Nodes (3): ClientCapabilities, ClientCapabilitiesDeps, OciClient

### Community 31 - "Community 31"
Cohesion: 0.15
Nodes (8): build_rekor_bundle(), build_sigstore_bundle_content(), CertificateVerifier, no_bundle_rejected_when_required(), reject_when_integrated_time_outside_cert_validity(), test_data(), verify_certificate_(), verify_with_valid_bundle()

### Community 32 - "Community 32"
Cohesion: 0.14
Nodes (14): decode_hex(), InclusionProof, Attestation, decode_body(), from_str_rejects_invalid_base64_body(), from_str_rejects_non_string_body(), InclusionProof, LOG_ENTRY (+6 more)

### Community 33 - "Community 33"
Cohesion: 0.14
Nodes (13): COSIGN_SIGN_V1_PREDICATE_TYPE, decode_in_toto_statement_from_real_bundle(), IN_TOTO_STATEMENT_V1_TYPE, InTotoStatementV1, REAL_BUNDLE_V03, statement_with_subjects(), Subject, subject_digests_yields_every_subject() (+5 more)

### Community 34 - "Community 34"
Cohesion: 0.13
Nodes (18): CONSISTENCY_PROOFS, ConsistencyProbe, ConsistencyTestVector, corrupt_consistency_proof(), corrupt_inclusion_proof(), INCLUSION_PROOFS, InclusionProbe, InclusionProofTestVector (+10 more)

### Community 35 - "Community 35"
Cohesion: 0.17
Nodes (9): MESSAGE, DEFAULT_KEY_SIZE, DigestAlgorithm, Sha256, PaddingScheme, PKCS1v15, PSS, rsa_schemes() (+1 more)

### Community 36 - "Community 36"
Cohesion: 0.13
Nodes (15): scheme_alg(), verify_alg(), verify_spki(), verify_spki_prehash(), backend_accepts_key(), VerifyScheme, EcdsaP256Sha256, EcdsaP384Sha384 (+7 more)

### Community 37 - "Community 37"
Cohesion: 0.20
Nodes (12): build_correct_bundle(), Bundle, bundle_new_verified_failure_because_different_key_signed_the_bundle(), bundle_new_verified_failure_because_rekor_key_id_is_unknown(), bundle_new_verified_success(), BundleContent, RekorBundle, SigstoreBundle (+4 more)

### Community 38 - "Community 38"
Cohesion: 0.16
Nodes (9): MerkleProofError, IndexGtTreeSize, MismatchedRoot, NewTreeSmaller, UnexpectedEmptyProof, UnexpectedNonEmptyProof, WrongEmptyTreeHash, WrongProofSize (+1 more)

### Community 39 - "Community 39"
Cohesion: 0.16
Nodes (6): cache_dir(), is_timerange_valid(), test_is_timerange_valid(), trust_root_fetch(), trust_root_outdated(), verify()

### Community 41 - "Community 41"
Cohesion: 0.15
Nodes (3): Claims, IdentityToken, UnverifiedClaims

### Community 42 - "Community 42"
Cohesion: 0.27
Nodes (13): cert_public_key(), cert_verify(), ec_from_point(), ec_point_key(), exponent_accepted(), key_matches_scheme(), p256_from_point(), public_key() (+5 more)

### Community 43 - "Community 43"
Cohesion: 0.11
Nodes (18): CertAlg, EcdsaP256Sha256, EcdsaP256Sha384, EcdsaP256Sha512, EcdsaP384Sha256, EcdsaP384Sha384, EcdsaP384Sha512, EcdsaP521Sha256 (+10 more)

### Community 44 - "Community 44"
Cohesion: 0.13
Nodes (13): search_index(), SearchIndexError, DefaultResponse, Status400, UnknownValue, Format, Minisign, Pgp (+5 more)

### Community 45 - "Community 45"
Cohesion: 0.18
Nodes (3): SigstoreTrustRoot, test_load_missing_trust_client_config(), trust_root()

### Community 46 - "Community 46"
Cohesion: 0.26
Nodes (14): is_trusted(), verify_cert_expiration_failure(), verify_cert_expiration_success(), verify_cert_failure_because_no_san(), verify_cert_key_usages_failure_because_no_code_signing(), verify_cert_key_usages_failure_because_no_digital_signature(), verify_cert_key_usages_success(), verify_cert_validity_failure() (+6 more)

### Community 48 - "Community 48"
Cohesion: 0.15
Nodes (7): client_builder_parses_ed25519_rekor_key(), ClientBuilder, ClientBuilder<'a>, OLD_REKOR_ED25519_KEY_DER, OLD_REKOR_ED25519_KEY_ID, ClientConfig, oci_client::client::ClientConfig

### Community 49 - "Community 49"
Cohesion: 0.16
Nodes (8): get_fulcio_cert_pool(), build_signature_layers(), get_bundle_from_annotations_works(), get_certificate_signature_from_annotations_fails_when_no_bundle_is_given(), get_certificate_signature_from_annotations_returns_none(), new_signature_layer_fails_because_bad_descriptor(), new_signature_layer_fails_because_bad_layer(), new_signature_layer_fails_because_checksum_mismatch()

### Community 50 - "Community 50"
Cohesion: 0.26
Nodes (3): SigningContext, SigningSession, Keyring

### Community 51 - "Community 51"
Cohesion: 0.20
Nodes (13): DSSE_PAYLOAD_TYPE_IN_TOTO_JSON, ED25519, SIGSTORE_BUNDLE_ANNOTATION, SIGSTORE_BUNDLE_V03_MEDIA_TYPE, SIGSTORE_CERT_ANNOTATION, SIGSTORE_GITHUB_WORKFLOW_NAME_OID, SIGSTORE_GITHUB_WORKFLOW_REF_OID, SIGSTORE_GITHUB_WORKFLOW_REPOSITORY_OID (+5 more)

### Community 52 - "Community 52"
Cohesion: 0.19
Nodes (15): CertificateValidationError, Extension, Malformed, NotCA, NotLeaf, VersionUnsupported, ExtensionErrorKind, BitUnset (+7 more)

### Community 54 - "Community 54"
Cohesion: 0.16
Nodes (4): Critical, CRITICAL_TYPE_NAME, Identity, Image

### Community 55 - "Community 55"
Cohesion: 0.15
Nodes (4): Version, Bundle0_1, Bundle0_2, Bundle0_3

### Community 56 - "Community 56"
Cohesion: 0.23
Nodes (7): Signer, SigStoreSigner, ECDSA_P256_SHA256_ASN1, ECDSA_P384_SHA384_ASN1, ECDSA_P521_SHA512_ASN1, RSA_PKCS1_SHA256, RSA_PSS_SHA256

### Community 57 - "Community 57"
Cohesion: 0.14
Nodes (11): ProposedEntry, Alpine, Hashedrekord, Helm, Intoto, Jar, Rekord, Rfc3161 (+3 more)

### Community 59 - "Community 59"
Cohesion: 0.21
Nodes (3): ManualTrustRoot, ManualTrustRoot<'a>, TrustRoot

### Community 60 - "Community 60"
Cohesion: 0.21
Nodes (8): CertificateEncoding, Der, ClientProtocol, Http, Https, HttpsExcept, oci_client::client::CertificateEncoding, oci_client::client::ClientProtocol

### Community 61 - "Community 61"
Cohesion: 0.23
Nodes (11): get_log_info(), get_log_proof(), GetLogInfoError, ConversionError, DefaultResponse, UnknownValue, GetLogProofError, ConversionError (+3 more)

### Community 62 - "Community 62"
Cohesion: 0.31
Nodes (5): hex_to_hash_output(), Rfc6269Default, ConsistencyProofError, InclusionProofError, UnexpectedError

### Community 63 - "Community 63"
Cohesion: 0.25
Nodes (6): certificate_signature_from_der_certificate(), certificate_signature_from_der_certificate_rejects_bad_der(), CertificateSignature, CertificateSubject, Email, Uri

### Community 64 - "Community 64"
Cohesion: 0.29
Nodes (8): Error, Io, Reqwest, ResponseError, Serde, error_status(), ResponseContent, urlencode()

### Community 65 - "Community 65"
Cohesion: 0.31
Nodes (5): CertificateEmbeddedSCT, SCTError, Parsing, Serialization, Verification

### Community 67 - "Community 67"
Cohesion: 0.29
Nodes (5): cert_is_preissuer(), find_issuer_cert(), Certificate, CertificateDer<'a>, oci_client::client::Certificate

### Community 68 - "Community 68"
Cohesion: 0.28
Nodes (4): Rfc6269HashPrefix, RFC6962LeafHashPrefix, RFC6962NodeHashPrefix, TestCase

### Community 69 - "Community 69"
Cohesion: 0.25
Nodes (4): CosignCapabilities, prepare_image_to_be_signed(), registry_image(), sign_verify_image()

### Community 72 - "Community 72"
Cohesion: 0.28
Nodes (7): DigitallySigned, SignatureType, CertificateTimestamp, TreeHash, SignedEntry, PrecertEntry, X509Entry

### Community 73 - "Community 73"
Cohesion: 0.32
Nodes (7): CertificateErrorKind, Der, IssuerMalformed, IssuerMissing, LeafSCTMalformed, LeafSCTMissing, Tls

### Community 74 - "Community 74"
Cohesion: 0.29
Nodes (5): get_public_key(), GetPublicKeyError, DefaultResponse, UnknownValue, Error

### Community 75 - "Community 75"
Cohesion: 0.38
Nodes (4): OpenSslAlgorithm, RSA_PKCS1_SHA256_ABSENT, RSA_PKCS1_SHA384_ABSENT, RSA_PKCS1_SHA512_ABSENT

### Community 77 - "Community 77"
Cohesion: 0.47
Nodes (5): SignConstraintRefVec, VerificationConstraintRefVec, Result, SigstoreApplicationConstraintsError, SigstoreVerifyConstraintsError

### Community 78 - "Community 78"
Cohesion: 0.40
Nodes (6): from_sigstore_bundle_rejects_unexpected_types(), mutated_v3_bundle_fixture(), V3BundleMutation, Payload, Predicate, Statement

### Community 80 - "Community 80"
Cohesion: 0.40
Nodes (5): CertificateErrorKind, Expired, Malformed, Sct, VerificationFailed

### Community 82 - "Community 82"
Cohesion: 0.40
Nodes (5): KeyringError, AlgoUnsupported, KeyMalformed, KeyNotFound, VerificationFailed

### Community 83 - "Community 83"
Cohesion: 0.50
Nodes (4): NotCAErrorKind, Invalid, NotCA, NotRootCA

### Community 85 - "Community 85"
Cohesion: 0.50
Nodes (3): impl_static_resource!, SIGSTORE_METADATA_BASE, SIGSTORE_TARGET_BASE

### Community 86 - "Community 86"
Cohesion: 0.67
Nodes (3): LogEntryType, PrecertEntry, X509Entry

## Knowledge Gaps
- **255 isolated node(s):** `ApplyConstraintError`, `Base64DecodeError`, `CertificateExpiredBeforeSignaturesSubmittedToRekor`, `CertificateInvalidEmail`, `CertificateIssuedAfterSignaturesSubmittedToRekor` (+250 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 635 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **13 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `SigstoreError` connect `Community 0` to `Community 3`, `Community 6`, `Community 7`, `Community 8`, `Community 11`, `Community 13`, `Community 20`, `Community 21`, `Community 23`, `Community 27`, `Community 28`, `Community 29`, `Community 30`, `Community 31`, `Community 32`, `Community 33`, `Community 37`, `Community 38`, `Community 39`, `Community 40`, `Community 41`, `Community 46`, `Community 47`, `Community 53`, `Community 62`, `Community 65`, `Community 77`, `Community 82`?**
  _High betweenness centrality (0.173) - this node is a cross-community bridge._
- **Are the 5 inferred relationships involving `SigstoreError` (e.g. with `SignatureErrorKind` and `material_from_private()`) actually correct?**
  _`SigstoreError` has 5 INFERRED edges - model-reasoned connections that need verification._
- **What connects `ApplyConstraintError`, `Base64DecodeError`, `CertificateExpiredBeforeSignaturesSubmittedToRekor` to the rest of the system?**
  _255 weakly-connected nodes found - possible missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.02564102564102564 - nodes in this community are weakly interconnected._
- **Why does `CosignVerificationKey` connect `Community 1` to `Community 3`, `Community 4`, `Community 10`, `Community 11`, `Community 12`, `Community 13`, `Community 14`, `Community 18`, `Community 19`, `Community 23`, `Community 25`, `Community 27`, `Community 28`, `Community 31`, `Community 32`, `Community 35`, `Community 36`, `Community 37`, `Community 48`, `Community 49`, `Community 56`, `Community 62`, `Community 63`?**
  _High betweenness centrality (0.099) - this node is a cross-community bridge._
- **Are the 57 inferred relationships involving `Value` (e.g. with `InTotoStatementV1` and `Subject`) actually correct?**
  _`Value` has 57 INFERRED edges - model-reasoned connections that need verification._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.05878332194121668 - nodes in this community are weakly interconnected._