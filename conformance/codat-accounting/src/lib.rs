//! Generated Codat Accounting API client — conformance crate.
//!
//! Tests pin that deep JSON-pointer $refs are hoisted to shared types.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn account_ref_round_trips() {
        let payload = json!({
            "id": "1b6266d1-1e44-46c5-8eb5-a8f98e03124e",
            "name": "Accounts Receivable"
        });
        let account_ref: types::AccountRef =
            serde_json::from_value(payload.clone()).expect("accountRef deserializes");
        assert_eq!(
            account_ref.id.as_deref(),
            Some("1b6266d1-1e44-46c5-8eb5-a8f98e03124e")
        );
        assert_eq!(account_ref.name.as_deref(), Some("Accounts Receivable"));
        assert_eq!(serde_json::to_value(&account_ref).expect("serializes"), payload);
    }

    #[test]
    fn account_ref_fields_are_optional_per_spec() {
        let empty: types::AccountRef = serde_json::from_value(json!({})).expect("empty ok");
        assert_eq!(empty.id, None);
        assert_eq!(empty.name, None);
        assert_eq!(serde_json::to_value(&empty).unwrap(), json!({}));
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
    }
}
