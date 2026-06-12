//! Generated MongoDB Atlas API client — conformance crate.
//!
//! Tests pin the cyclic oneOf+discriminator(providerName) construct where
//! each variant allOf-refs the base back; the prepass must fold base
//! properties into every subtype without stack-overflowing.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    fn aws_container_payload() -> serde_json::Value {
        serde_json::json!({
            "providerName": "AWS",
            "id": "32b6e34b3d91647abb20e7b8",
            "provisioned": true,
            "atlasCidrBlock": "10.8.0.0/21",
            "regionName": "US_EAST_1"
        })
    }

    #[test]
    fn union_payload_with_aws_discriminator_picks_aws_variant() {
        let container: types::CloudProviderContainer =
            serde_json::from_value(aws_container_payload()).expect("deserializes");

        let id = match &container {
            types::CloudProviderContainer::Aws { id, .. } => id,
            other => panic!("AWS payload must pick AWS variant, got: {other:?}"),
        };
        assert_eq!(id.as_deref(), Some("32b6e34b3d91647abb20e7b8"));

        let round_tripped = serde_json::to_value(&container).expect("serializes");
        assert_eq!(round_tripped, aws_container_payload());
    }

    #[test]
    fn base_properties_are_present_on_the_aws_variant() {
        let container: types::CloudProviderContainer =
            serde_json::from_value(aws_container_payload()).expect("deserializes");

        match &container {
            types::CloudProviderContainer::Aws { provisioned, provider_name, .. } => {
                assert_eq!(*provisioned, Some(true));
                assert_eq!(provider_name.as_deref(), Some("AWS"));
            }
            _ => panic!("wrong variant"),
        }
    }

    #[test]
    fn azure_discriminator_routes_to_azure_variant() {
        let azure_payload = serde_json::json!({
            "providerName": "AZURE",
            "id": "abc123",
            "provisioned": false,
            "atlasCidrBlock": "192.168.0.0/21",
            "azureSubscriptionId": "sub-001",
            "region": "US_EAST_2"
        });
        let container: types::CloudProviderContainer =
            serde_json::from_value(azure_payload.clone()).expect("Azure payload deserializes");

        assert!(
            matches!(container, types::CloudProviderContainer::Azure { .. }),
            "Azure payload must select the Azure variant"
        );
    }

    #[test]
    fn provider_name_guards_documented_discriminants() {
        for name in ["AWS", "GCP", "AZURE", "TENANT", "SERVERLESS"] {
            assert!(
                types::CloudProviderContainerProviderName::from_str(name).is_ok(),
                "{name} must be a documented providerName"
            );
        }
        assert!(
            types::CloudProviderContainerProviderName::from_str("BOGUS").is_err(),
            "undocumented providerName must be rejected"
        );
    }
}
