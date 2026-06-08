use crate::services::{oneshot_relayer, venice_agent};
use sqlx::PgPool;
use std::sync::Arc;
use uuid::Uuid;

pub async fn process_pr_event(pool: Arc<PgPool>, pr_event_id: Uuid) -> anyhow::Result<()> {
    let result = process_inner(pool.clone(), pr_event_id).await;
    if let Err(ref e) = result {
        tracing::error!("Payment processor error for pr_event {}: {:?}", pr_event_id, e);
        // Best-effort: mark as failed. Ignored if the DB is also down.
        let _ = sqlx::query("UPDATE pr_events SET status = 'failed' WHERE id = $1")
            .bind(pr_event_id)
            .execute(pool.as_ref())
            .await;
    }
    result
}

async fn process_inner(pool: Arc<PgPool>, pr_event_id: Uuid) -> anyhow::Result<()> {
    // 1. Fetch PrEvent
    let pr_event = sqlx::query_as::<_, (i32, String, String, Uuid)>(
        "SELECT pr_number, pr_title, contributor, repo_id FROM pr_events WHERE id = $1",
    )
    .bind(pr_event_id)
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| anyhow::anyhow!("PrEvent {} not found", pr_event_id))?;

    let (pr_number, pr_title, contributor, repo_id) = pr_event;

    // 2. Fetch Repo
    let repo = sqlx::query_as::<_, (f64, f64, Option<String>)>(
        "SELECT payout_amount, daily_cap, delegation_hex FROM repos WHERE id = $1",
    )
    .bind(repo_id)
    .fetch_optional(pool.as_ref())
    .await?
    .ok_or_else(|| anyhow::anyhow!("Repo {} not found", repo_id))?;

    let (payout_amount, daily_cap, delegation_hex) = repo;

    // 3. Fetch Contributor wallet
    let contributor_wallet: Option<String> = sqlx::query_scalar(
        "SELECT wallet_address FROM contributors WHERE github_username = $1 AND repo_id = $2",
    )
    .bind(&contributor)
    .bind(repo_id)
    .fetch_optional(pool.as_ref())
    .await?;

    // 4. No contributor registered — reject cleanly, no error
    let wallet_address = match contributor_wallet {
        Some(w) => w,
        None => {
            tracing::warn!(
                "No registered contributor for github_username={} in repo={}",
                contributor,
                repo_id
            );
            sqlx::query("UPDATE pr_events SET status = 'rejected' WHERE id = $1")
                .bind(pr_event_id)
                .execute(pool.as_ref())
                .await?;
            return Ok(());
        }
    };

    // 5. No delegation — reject cleanly, no error
    let delegation_hex = match delegation_hex {
        Some(d) => d,
        None => {
            tracing::warn!("No delegation_hex set for repo {}", repo_id);
            sqlx::query("UPDATE pr_events SET status = 'rejected' WHERE id = $1")
                .bind(pr_event_id)
                .execute(pool.as_ref())
                .await?;
            return Ok(());
        }
    };

    // 6. Calculate daily_spent — propagate DB errors instead of silently defaulting to 0
    let daily_spent: f64 = sqlx::query_scalar::<_, f64>(
        "SELECT COALESCE(SUM(p.amount), 0.0::float8) \
         FROM payments p JOIN pr_events pe ON p.pr_event_id = pe.id \
         WHERE pe.repo_id = $1 AND p.created_at > NOW() - INTERVAL '1 day'",
    )
    .bind(repo_id)
    .fetch_one(pool.as_ref())
    .await?;

    // 7. Ask Venice AI
    let decision = venice_agent::evaluate_pr(
        &pr_title,
        &contributor,
        payout_amount,
        daily_cap,
        daily_spent,
    )
    .await?;

    tracing::info!(
        "Venice decision for PR #{}: approved={}, amount={}, reason={}",
        pr_number,
        decision.approved,
        decision.amount,
        decision.reason
    );

    // 8. Not approved — insert rejection record + update status atomically
    if !decision.approved {
        let mut tx = pool.begin().await?;
        sqlx::query(
            "INSERT INTO payments (pr_event_id, amount, venice_reason) VALUES ($1, 0.0, $2)",
        )
        .bind(pr_event_id)
        .bind(&decision.reason)
        .execute(&mut *tx)
        .await?;
        sqlx::query("UPDATE pr_events SET status = 'rejected' WHERE id = $1")
            .bind(pr_event_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        return Ok(());
    }

    // 9. Daily cap check — insert rejection record + update status atomically
    if daily_spent + decision.amount > daily_cap {
        tracing::warn!(
            "Daily cap exceeded for repo {}: spent={}, requested={}, cap={}",
            repo_id,
            daily_spent,
            decision.amount,
            daily_cap
        );
        let mut tx = pool.begin().await?;
        sqlx::query(
            "INSERT INTO payments (pr_event_id, amount, venice_reason) VALUES ($1, 0.0, $2)",
        )
        .bind(pr_event_id)
        .bind("Daily cap exceeded")
        .execute(&mut *tx)
        .await?;
        sqlx::query("UPDATE pr_events SET status = 'rejected' WHERE id = $1")
            .bind(pr_event_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        return Ok(());
    }

    // 10. Validate amount before calling the relayer
    if decision.amount <= 0.0 {
        tracing::warn!(
            "Venice returned non-positive amount {} for PR #{}, rejecting",
            decision.amount,
            pr_number
        );
        let mut tx = pool.begin().await?;
        sqlx::query(
            "INSERT INTO payments (pr_event_id, amount, venice_reason) VALUES ($1, 0.0, $2)",
        )
        .bind(pr_event_id)
        .bind("Invalid amount from AI")
        .execute(&mut *tx)
        .await?;
        sqlx::query("UPDATE pr_events SET status = 'rejected' WHERE id = $1")
            .bind(pr_event_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        return Ok(());
    }

    // 11. Execute transfer — record outcome atomically regardless of relay result
    match oneshot_relayer::execute_usdc_transfer(&wallet_address, decision.amount, &delegation_hex)
        .await
    {
        Ok(tx_hash) => {
            let mut tx = pool.begin().await?;
            sqlx::query(
                "INSERT INTO payments (pr_event_id, amount, tx_hash, venice_reason) \
                 VALUES ($1, $2, $3, $4)",
            )
            .bind(pr_event_id)
            .bind(decision.amount)
            .bind(&tx_hash)
            .bind(&decision.reason)
            .execute(&mut *tx)
            .await?;
            sqlx::query("UPDATE pr_events SET status = 'paid' WHERE id = $1")
                .bind(pr_event_id)
                .execute(&mut *tx)
                .await?;
            tx.commit().await?;
            tracing::info!(
                "Payment successful: {} USDC to {} | tx={}",
                decision.amount,
                wallet_address,
                tx_hash
            );
        }
        Err(e) => {
            // Relay failed — update status without propagating as a processor error,
            // since this is an expected external failure, not a logic bug.
            tracing::error!("1Shot relay failed for PR #{}: {:?}", pr_number, e);
            if let Err(db_err) =
                sqlx::query("UPDATE pr_events SET status = 'failed' WHERE id = $1")
                    .bind(pr_event_id)
                    .execute(pool.as_ref())
                    .await
            {
                tracing::error!("Also failed to update status to 'failed': {:?}", db_err);
            }
        }
    }

    Ok(())
}
