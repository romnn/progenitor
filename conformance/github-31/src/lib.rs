//! Generated github-31 API client — conformance crate.
//!
//! This client is one of the corpus monsters (hundreds of thousands of
//! generated lines); building it takes rustc a long time on stable. The
//! nightly parallel frontend helps:
//! `RUSTFLAGS="-Z threads=8" cargo +nightly check -p conformance-github-31`.

include!(concat!(env!("OUT_DIR"), "/codegen.rs"));

#[cfg(test)]
mod tests {
    use super::*;

    conformance_support::assert_send_sync!(Client);
    conformance_support::assert_display!(types::ReactionContent);
    conformance_support::assert_from_str!(types::ReactionContent);

    #[test]
    fn client_constructs() {
        let client = Client::new("https://example.invalid");
        assert_eq!(client.baseurl(), "https://example.invalid");
    }

    #[test]
    fn reaction_rollup_roundtrip() {
        conformance_support::roundtrip!(
            types::ReactionRollup,
            conformance_support::serde_json::json!({
                "+1": 5,
                "-1": 0,
                "confused": 1,
                "eyes": 0,
                "heart": 3,
                "hooray": 0,
                "laugh": 0,
                "rocket": 2,
                "total_count": 11,
                "url": "https://api.github.com/repos/owner/repo/issues/1/reactions"
            })
        );
    }

    #[test]
    fn reaction_content_wire_names() {
        // +1 and -1 are renamed variants — verify the serde rename roundtrips correctly
        conformance_support::assert_wire_enum!(
            types::ReactionContent,
            "+1" => types::ReactionContent::Plus1,
            "-1" => types::ReactionContent::Minus1,
            "laugh" => types::ReactionContent::Laugh,
            "heart" => types::ReactionContent::Heart,
        );
    }

    #[test]
    fn org_rules_variants() {
        conformance_support::assert_union_variants!(
            types::OrgRules,
            conformance_support::serde_json::json!({"type": "creation"}) => types::OrgRules::Creation,
            conformance_support::serde_json::json!({"type": "deletion"}) => types::OrgRules::Deletion,
        );
    }

    #[test]
    fn unknown_reaction_rejected() {
        conformance_support::assert_rejects!(
            types::ReactionContent,
            conformance_support::serde_json::json!("clapping"),
            "ReactionContent has no variant for unknown wire value"
        );
    }
}

#[cfg(test)]
mod example_tests {
    include!(concat!(env!("OUT_DIR"), "/example_tests.rs"));
}

#[cfg(test)]
pub mod mock {
    include!(concat!(env!("OUT_DIR"), "/mock.rs"));
}

#[cfg(test)]
mod operation_tests {
    use super::*;
    use conformance_support::httpmock::MockServer;

    #[conformance_support::tokio::test]
    async fn meta_root_sends_get_to_root_path() {
        let server = MockServer::start_async().await;
        let mock = server
            .mock_async(|when, then| {
                when.method("GET").path("/");
                then.status(200).json_body(serde_json::json!({
                    "authorizations_url": "https://api.github.com/authorizations",
                    "code_search_url": "https://api.github.com/search/code?q={query}",
                    "commit_search_url": "https://api.github.com/search/commits?q={query}",
                    "current_user_authorizations_html_url": "https://github.com/settings/connections/applications{/client_id}",
                    "current_user_repositories_url": "https://api.github.com/user/repos{?type,page,per_page,sort}",
                    "current_user_url": "https://api.github.com/user",
                    "emails_url": "https://api.github.com/user/emails",
                    "emojis_url": "https://api.github.com/emojis",
                    "events_url": "https://api.github.com/events",
                    "feeds_url": "https://api.github.com/feeds",
                    "followers_url": "https://api.github.com/user/followers",
                    "following_url": "https://api.github.com/user/following{/target}",
                    "gists_url": "https://api.github.com/gists{/gist_id}",
                    "issue_search_url": "https://api.github.com/search/issues?q={query}",
                    "issues_url": "https://api.github.com/issues",
                    "keys_url": "https://api.github.com/user/keys",
                    "label_search_url": "https://api.github.com/search/labels?q={query}",
                    "notifications_url": "https://api.github.com/notifications",
                    "organization_repositories_url": "https://api.github.com/orgs/{org}/repos{?type,page,per_page,sort}",
                    "organization_teams_url": "https://api.github.com/orgs/{org}/teams",
                    "organization_url": "https://api.github.com/orgs/{org}",
                    "public_gists_url": "https://api.github.com/gists/public",
                    "rate_limit_url": "https://api.github.com/rate_limit",
                    "repository_search_url": "https://api.github.com/search/repositories?q={query}",
                    "repository_url": "https://api.github.com/repos/{owner}/{repo}",
                    "starred_gists_url": "https://api.github.com/gists/starred",
                    "starred_url": "https://api.github.com/user/starred{/owner}{/repo}",
                    "user_organizations_url": "https://api.github.com/user/orgs",
                    "user_repositories_url": "https://api.github.com/users/{user}/repos{?type,page,per_page,sort}",
                    "user_search_url": "https://api.github.com/search/users?q={query}",
                    "user_url": "https://api.github.com/users/{user}"
                }));
            })
            .await;
        let client = Client::new(&server.base_url());
        let result = client.meta_root().await;
        mock.assert_async().await;
        assert!(result.is_ok(), "expected 200 success from mock");
    }
}
