//! Behavioral checks for the codat-accounting client. The spec nests
//! reusable schemas in non-standard `definitions` blocks inside component
//! schemas and refs them with deep pointers
//! (`#/components/schemas/Account/definitions/accountRef`, 15 sites;
//! `PushOption/definitions/pushOptionProperty`, which is self-referential).
//! Generation used to fail on those pointers; these tests pin that the
//! hoisted types are shared, recursive where the spec recurses, and
//! faithful to the documented wire format.

use serde_json::json;
use wild_codat_accounting::types;

#[test]
fn account_ref_round_trips() {
    let payload = json!({
        "id": "1b6266d1-1e44-46c5-8eb5-a8f98e03124e",
        "name": "Accounts Receivable"
    });
    let account_ref: types::AccountRef =
        serde_json::from_value(payload.clone()).expect("accountRef payload deserializes");
    assert_eq!(
        account_ref.id.as_deref(),
        Some("1b6266d1-1e44-46c5-8eb5-a8f98e03124e")
    );
    assert_eq!(account_ref.name.as_deref(), Some("Accounts Receivable"));

    // Both documented fields must survive the round-trip; a silently
    // dropped field makes this Value comparison fail.
    let round_tripped = serde_json::to_value(&account_ref).expect("serializes");
    assert_eq!(round_tripped, payload);
}

#[test]
fn account_ref_fields_are_optional_per_spec() {
    // Neither `id` nor `name` is in a `required` list.
    let empty: types::AccountRef = serde_json::from_value(json!({})).expect("empty object ok");
    assert_eq!(empty.id, None);
    assert_eq!(empty.name, None);
    assert_eq!(serde_json::to_value(&empty).unwrap(), json!({}));
}

// Compile-time proof of type-level sharing: distinct parent schemas that
// `$ref` `Account/definitions/accountRef` must carry the SAME hoisted type,
// not per-site clones.
fn journal_line_account_ref(line: &types::JournalLine) -> Option<&types::AccountRef> {
    line.account_ref.as_ref()
}

fn bill_credit_note_account_ref(line: &types::BillCreditNoteLineItem) -> Option<&types::AccountRef> {
    line.account_ref.as_ref()
}

#[test]
fn parents_share_the_hoisted_account_ref_type() {
    let journal_line: types::JournalLine = serde_json::from_value(json!({
        "netAmount": 250.5,
        "accountRef": {"id": "acc-001", "name": "Sales"}
    }))
    .expect("journal line deserializes");
    let shared = journal_line_account_ref(&journal_line)
        .expect("accountRef present")
        .clone();
    assert_eq!(shared.id.as_deref(), Some("acc-001"));

    // Move the very same value into a different parent type: only possible
    // because both fields are Option<AccountRef> of one shared type.
    let mut bill_line: types::BillCreditNoteLineItem =
        serde_json::from_value(json!({"quantity": 2.0, "unitAmount": 7.5}))
            .expect("bill credit note line deserializes");
    assert!(bill_credit_note_account_ref(&bill_line).is_none());
    bill_line.account_ref = Some(shared);

    let value = serde_json::to_value(&bill_line).expect("serializes");
    assert_eq!(value["accountRef"]["id"], "acc-001");
    assert_eq!(value["accountRef"]["name"], "Sales");
}

#[test]
fn journal_line_constructed_with_account_ref_uses_documented_wire_name() {
    let line = types::JournalLine {
        account_ref: Some(types::AccountRef {
            id: Some("acc-610".to_string()),
            name: Some("Accounts Receivable".to_string()),
        }),
        contact_ref: None,
        currency: None,
        description: None,
        net_amount: 99.5,
        tracking: None,
        transaction_amount: None,
        transaction_currency: None,
    };
    let value = serde_json::to_value(&line).expect("serializes");
    // camelCase wire names from the spec, not the Rust field names.
    assert_eq!(value["accountRef"], json!({"id": "acc-610", "name": "Accounts Receivable"}));
    assert_eq!(value["netAmount"], json!(99.5));
}

#[test]
fn push_option_property_recurses_two_levels_through_serde() {
    // PushOption.properties values are pushOptionProperty, which contains
    // `properties: map<string, pushOptionProperty>` — self-referential.
    let payload = json!({
        "displayName": "Bill",
        "required": true,
        "type": "Object",
        "properties": {
            "lineItems": {
                "description": "Lines on the bill.",
                "displayName": "Line items",
                "required": true,
                "type": "Array",
                "properties": {
                    "accountRef": {
                        "description": "Linked ledger account.",
                        "displayName": "Account",
                        "required": false,
                        "type": "String",
                        "options": [{"value": "0123", "type": "String"}],
                        "validation": {
                            "warnings": [{
                                "field": "accountRef",
                                "details": "Account must exist on the platform."
                            }]
                        }
                    }
                }
            }
        }
    });

    let option: types::PushOption =
        serde_json::from_value(payload.clone()).expect("recursive push option deserializes");

    let level1 = &option.properties.as_ref().expect("level-1 properties")["lineItems"];
    assert_eq!(level1.display_name.as_str(), "Line items");
    assert_eq!(level1.description.0, "Lines on the bill.");
    assert!(level1.required.0);
    assert_eq!(level1.type_, types::PushOptionType::Array);

    let level2 = &level1.properties.as_ref().expect("level-2 properties")["accountRef"];
    assert_eq!(level2.display_name.as_str(), "Account");
    assert!(!level2.required.0);
    assert_eq!(level2.type_, types::PushOptionType::String);
    let choice = &level2.options.as_ref().expect("options")[0];
    assert_eq!(choice.value.as_ref().expect("choice value").as_str(), "0123");
    let warning = &level2.validation.as_ref().expect("validation").warnings.as_ref().unwrap()[0];
    assert_eq!(warning.field.as_deref(), Some("accountRef"));
    assert_eq!(warning.details.as_str(), "Account must exist on the platform.");

    // Nothing documented may be dropped at any depth.
    let round_tripped = serde_json::to_value(&option).expect("serializes");
    assert_eq!(round_tripped, payload);
}

#[test]
fn push_option_property_constructs_recursively_in_rust() {
    let leaf = types::PushOptionProperty {
        description: types::PushOptionDescription("Leaf field.".to_string()),
        display_name: "Leaf".parse().expect("valid display name"),
        options: None,
        properties: None,
        required: types::Required(true),
        type_: types::PushOptionType::Boolean,
        validation: None,
    };
    let mid = types::PushOptionProperty {
        description: types::PushOptionDescription("Holds the leaf.".to_string()),
        display_name: "Mid".parse().expect("valid display name"),
        options: None,
        properties: Some([("leaf".to_string(), leaf)].into_iter().collect()),
        required: types::Required(false),
        type_: types::PushOptionType::Object,
        validation: None,
    };
    let root = types::PushOptionProperty {
        description: types::PushOptionDescription("Root.".to_string()),
        display_name: "Root".parse().expect("valid display name"),
        options: None,
        properties: Some([("mid".to_string(), mid)].into_iter().collect()),
        required: types::Required(true),
        type_: types::PushOptionType::Object,
        validation: None,
    };

    let value = serde_json::to_value(&root).expect("serializes");
    assert_eq!(value["properties"]["mid"]["properties"]["leaf"]["displayName"], "Leaf");
    assert_eq!(value["properties"]["mid"]["properties"]["leaf"]["type"], "Boolean");
    assert_eq!(value["properties"]["mid"]["properties"]["leaf"]["required"], true);
}

#[test]
fn push_option_property_required_fields_match_the_spec() {
    // pushOptionProperty requires description+displayName+required+type;
    // the parent PushOption requires only displayName+required+type.
    let missing_description = json!({
        "displayName": "X",
        "required": true,
        "type": "String"
    });
    assert!(
        serde_json::from_value::<types::PushOptionProperty>(missing_description.clone()).is_err(),
        "pushOptionProperty without `description` must be rejected"
    );
    assert!(
        serde_json::from_value::<types::PushOption>(missing_description).is_ok(),
        "PushOption itself documents `description` as optional"
    );

    let complete = json!({
        "description": "d",
        "displayName": "X",
        "required": true,
        "type": "String"
    });
    let property: types::PushOptionProperty =
        serde_json::from_value(complete.clone()).expect("complete property deserializes");
    assert_eq!(serde_json::to_value(&property).unwrap(), complete);
}

#[test]
fn push_option_type_guards_documented_values() {
    let documented = [
        ("Array", types::PushOptionType::Array),
        ("Object", types::PushOptionType::Object),
        ("String", types::PushOptionType::String),
        ("Number", types::PushOptionType::Number),
        ("Boolean", types::PushOptionType::Boolean),
        ("DateTime", types::PushOptionType::DateTime),
        ("File", types::PushOptionType::File),
        ("MultiPart", types::PushOptionType::MultiPart),
    ];
    for (text, expected) in documented {
        let parsed = types::PushOptionType::try_from(text)
            .unwrap_or_else(|_| panic!("documented value {text:?} must parse"));
        assert_eq!(parsed, expected);
        assert_eq!(serde_json::to_value(parsed).unwrap(), json!(text));
    }
    for bogus in ["Integer", "string", "datetime", ""] {
        assert!(
            types::PushOptionType::try_from(bogus).is_err(),
            "undocumented value {bogus:?} must be rejected"
        );
    }
    assert!(serde_json::from_value::<types::PushOptionType>(json!("Multipart")).is_err());
}

#[test]
fn display_name_enforces_min_length_from_the_deep_pointer() {
    // PushOption.displayName refs
    // `pushOptionProperty/properties/displayName` (minLength: 1); the
    // hoisted newtype must keep that constraint, including through serde.
    assert!("".parse::<types::DisplayName>().is_err());
    let name: types::DisplayName = "Account".parse().expect("non-empty ok");
    assert_eq!(name.as_str(), "Account");

    let rejected = serde_json::from_value::<types::PushOption>(json!({
        "displayName": "",
        "required": true,
        "type": "String"
    }));
    assert!(rejected.is_err(), "empty displayName must fail deserialization");
}
