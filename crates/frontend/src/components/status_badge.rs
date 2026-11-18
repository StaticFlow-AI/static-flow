//! Reusable status pill for admin tables.
//!
//! Centralizes status-to-tone mapping so every admin surface (tasks, wishes,
//! audit actions, gateway accounts/keys, …) reads consistently across light and
//! dark themes. Styling lives in `input.css` under `.status-badge` and is
//! refined inside `.adm-shell` by `admin.css`.

use yew::prelude::*;

/// Props for [`StatusBadge`].
#[derive(Properties, PartialEq)]
pub struct StatusBadgeProps {
    /// Raw status string (matched case-insensitively against known values).
    pub status: AttrValue,
    /// Optional display label; defaults to the status text itself.
    #[prop_or_default]
    pub label: Option<AttrValue>,
    /// Optional extra classes appended to the outer pill.
    #[prop_or_default]
    pub class: Classes,
}

/// Maps a status to its `(tone-class, fontawesome-icon)`.
fn tone_and_icon(status: &str) -> (&'static str, &'static str) {
    match status {
        "pending" | "queued" | "waiting" => ("status-badge--pending", "fa-clock"),
        "approved" | "approve" | "patch" | "patched" | "update" | "updated" => {
            ("status-badge--approved", "fa-circle-check")
        },
        "running" | "processing" | "in_progress" | "in-progress" | "streaming" | "retry"
        | "retrying" | "approve_and_run" => ("status-badge--running", "fa-spinner fa-spin"),
        "done" | "success" | "succeeded" | "completed" | "ingested" | "issued" | "active"
        | "enabled" | "ok" | "healthy" | "valid" | "published" | "publish" | "created"
        | "create" => ("status-badge--done", "fa-circle-check"),
        "failed" | "error" | "errored" | "invalid" | "delete" | "deleted" | "remove" => {
            ("status-badge--failed", "fa-circle-exclamation")
        },
        "rejected" | "reject" | "disabled" | "expired" | "cancelled" | "canceled" | "revoked"
        | "cleanup" | "reset" => ("status-badge--rejected", "fa-ban"),
        _ => ("status-badge--neutral", "fa-circle-info"),
    }
}

/// A consistent status pill: soft background, semantic tone, calm label.
#[function_component(StatusBadge)]
pub fn status_badge(props: &StatusBadgeProps) -> Html {
    let lower = props.status.to_lowercase();
    let (tone, icon) = tone_and_icon(lower.as_str());
    let label = props.label.clone().unwrap_or_else(|| props.status.clone());
    html! {
        <span class={classes!("status-badge", tone, props.class.clone())}>
            <i class={classes!("fas", icon, "status-badge__icon")} aria-hidden="true"></i>
            <span>{ label }</span>
        </span>
    }
}
