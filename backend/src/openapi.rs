use crate::{
    models::{
        contributor::{Contributor, CreateContributorRequest},
        payment::{Payment, PaymentStats, PaymentWithPr},
        repo::{CreateRepoRequest, Repo, RepoWithStats},
    },
    routes::{
        repos::UpdateDelegationRequest,
        test_trigger::{PrStatusResponse, TestTriggerRequest},
    },
};
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    info(
        title = "GrantBot API",
        version = "1.0.0",
        description = "AI-powered autonomous USDC payments for open-source contributors.\n\n\
            **Flow:** Register a repo → add contributor wallets → grant ERC-7715 delegation \
            → merge a PR → Venice AI evaluates → 1Shot relayer pays USDC on Sepolia."
    ),
    paths(
        crate::routes::repos::create_repo,
        crate::routes::repos::get_repo,
        crate::routes::repos::update_delegation,
        crate::routes::contributors::add_contributor,
        crate::routes::contributors::list_contributors,
        crate::routes::payments::list_payments,
        crate::routes::payments::get_stats,
        crate::routes::webhook::handle_github_webhook,
        crate::routes::test_trigger::trigger_pr,
        crate::routes::test_trigger::get_pr_status,
    ),
    components(
        schemas(
            Repo,
            CreateRepoRequest,
            RepoWithStats,
            UpdateDelegationRequest,
            Contributor,
            CreateContributorRequest,
            Payment,
            PaymentWithPr,
            PaymentStats,
            TestTriggerRequest,
            PrStatusResponse,
        )
    ),
    tags(
        (name = "Repos",        description = "Register repositories and manage spending delegations"),
        (name = "Contributors", description = "Map GitHub usernames to wallet addresses"),
        (name = "Payments",     description = "Query payment history and statistics"),
        (name = "Webhooks",     description = "GitHub webhook receiver — set this as your repo Payload URL"),
        (name = "Test",         description = "Simulate merged PRs and poll payment status without a real webhook"),
    )
)]
pub struct ApiDoc;
