//! Behavioral spot-checks for the mongodb-atlas client, pinning the
//! construct that used to break generation: `CloudProviderContainer` is
//! a cyclic oneOf+discriminator(providerName) whose sibling properties
//! (id, providerName, provisioned) sit NEXT TO the oneOf, while each
//! variant (AWS/Azure/GCP) allOf-refs the base back. The prepass strips
//! the backref but must fold the base's shared properties into every
//! subtype — a regression that silently drops id/provisioned must fail
//! these tests.

use std::str::FromStr;

use wild_mongodb_atlas::types;

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

    match &container {
        types::CloudProviderContainer::Aws {
            atlas_cidr_block,
            id,
            provisioned,
            region_name,
            vpc_id,
        } => {
            // The shared base fields must have been folded into the
            // variant and populated, not dropped by the backref strip.
            assert_eq!(
                id.as_ref().map(|id| id.as_str()),
                Some("32b6e34b3d91647abb20e7b8")
            );
            assert_eq!(*provisioned, Some(true));
            assert_eq!(
                atlas_cidr_block.as_ref().map(|cidr| cidr.as_str()),
                Some("10.8.0.0/21")
            );
            assert_eq!(region_name.to_string(), "US_EAST_1");
            assert_eq!(*vpc_id, None);
        }
        other => panic!("expected the AWS variant, got {other:?}"),
    }
}

#[test]
fn union_round_trip_keeps_shared_base_fields() {
    let container: types::CloudProviderContainer =
        serde_json::from_value(aws_container_payload()).expect("deserializes");
    let value = serde_json::to_value(&container).expect("serializes");

    // The discriminator and every shared base field must survive the
    // round-trip on the wire, byte-for-byte equal to the input.
    assert_eq!(value["providerName"], "AWS");
    assert_eq!(value["id"], "32b6e34b3d91647abb20e7b8");
    assert_eq!(value["provisioned"], true);
    assert_eq!(value["atlasCidrBlock"], "10.8.0.0/21");
    assert_eq!(value["regionName"], "US_EAST_1");
}

#[test]
fn standalone_aws_subtype_keeps_inherited_base_fields() {
    let aws: types::AwsCloudProviderContainer =
        serde_json::from_value(aws_container_payload()).expect("deserializes");

    assert_eq!(
        aws.id.as_ref().map(|id| id.as_str()),
        Some("32b6e34b3d91647abb20e7b8")
    );
    assert_eq!(aws.provisioned, Some(true));
    assert_eq!(
        aws.provider_name,
        types::AwsCloudProviderContainerProviderName::Aws
    );
    assert_eq!(
        aws.region_name,
        types::AwsCloudProviderContainerRegionName::UsEast1
    );

    let value = serde_json::to_value(&aws).expect("serializes");
    assert_eq!(value["providerName"], "AWS");
    assert_eq!(value["id"], "32b6e34b3d91647abb20e7b8");
    assert_eq!(value["provisioned"], true);
    assert_eq!(value["atlasCidrBlock"], "10.8.0.0/21");
    assert_eq!(value["regionName"], "US_EAST_1");
}

#[test]
fn union_payload_with_azure_discriminator_picks_azure_variant() {
    // Same shared fields, AZURE discriminator: dispatch must follow the
    // tag, and the folded base fields must populate this variant too.
    let payload = serde_json::json!({
        "providerName": "AZURE",
        "id": "32b6e34b3d91647abb20e7b8",
        "provisioned": false,
        "atlasCidrBlock": "10.8.0.0/21",
        "region": "US_EAST_2"
    });
    let container: types::CloudProviderContainer =
        serde_json::from_value(payload).expect("deserializes");

    match &container {
        types::CloudProviderContainer::Azure {
            id, provisioned, ..
        } => {
            assert_eq!(
                id.as_ref().map(|id| id.as_str()),
                Some("32b6e34b3d91647abb20e7b8")
            );
            assert_eq!(*provisioned, Some(false));
        }
        other => panic!("expected the AZURE variant, got {other:?}"),
    }
}

#[test]
fn standalone_aws_subtype_rejects_foreign_discriminator() {
    // The discriminator mapping narrows the subtype's providerName to
    // its own tag; a GCP tag must not deserialize into the AWS struct.
    let mut payload = aws_container_payload();
    payload["providerName"] = "GCP".into();
    let result = serde_json::from_value::<types::AwsCloudProviderContainer>(payload);
    assert!(result.is_err(), "AWS subtype accepted providerName GCP");
}

#[test]
fn union_rejects_unmapped_discriminator_values() {
    // The base enum documents TENANT/SERVERLESS, but the discriminator
    // mapping covers only AWS/AZURE/GCP — no oneOf branch matches.
    for tag in ["TENANT", "SERVERLESS", "IBM"] {
        let mut payload = aws_container_payload();
        payload["providerName"] = tag.into();
        let result = serde_json::from_value::<types::CloudProviderContainer>(payload);
        assert!(result.is_err(), "union accepted providerName {tag}");
    }
}

#[test]
fn id_newtype_enforces_documented_pattern() {
    // id is constrained to ^([a-f0-9]{24})$ in the base schema; the
    // folded copy in the subtype must keep the guard.
    let id = types::AwsCloudProviderContainerId::from_str("32b6e34b3d91647abb20e7b8")
        .expect("documented example id parses");
    assert_eq!(id.as_str(), "32b6e34b3d91647abb20e7b8");

    for bad in ["", "not-hex", "32B6E34B3D91647ABB20E7B8", "32b6e34b"] {
        assert!(
            types::AwsCloudProviderContainerId::from_str(bad).is_err(),
            "id pattern accepted {bad:?}"
        );
        assert!(
            types::CloudProviderContainerId::try_from(bad).is_err(),
            "union id pattern accepted {bad:?}"
        );
    }
}

#[test]
fn region_name_rejects_undocumented_values() {
    let mut payload = aws_container_payload();
    payload["regionName"] = "MARS_CENTRAL_1".into();
    let result = serde_json::from_value::<types::CloudProviderContainer>(payload);
    assert!(result.is_err(), "union accepted an undocumented regionName");
}
