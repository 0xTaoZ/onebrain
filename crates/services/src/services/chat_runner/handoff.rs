//! Team handoffs: what a member that is new to a session needs from the others.
//!
//! Built from data the session already stores (user requests, each member's
//! latest conclusion and changed files, shared records), so it costs no extra
//! model call. It replaces reading the whole history file on a first run; the
//! file stays available when the handoffs are not enough.

use super::*;

const HANDOFF_USER_REQUESTS: usize = 2;
const HANDOFF_REQUEST_CHARS: usize = 1_500;
const HANDOFF_CONCLUSION_CHARS: usize = 800;
const HANDOFF_MAX_FILES: usize = 20;
const HANDOFF_RECORDS: usize = 10;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MemberHandoff {
    pub(super) name: String,
    pub(super) conclusion: Option<String>,
    pub(super) files: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct HandoffBrief {
    /// Latest user requests, oldest first, verbatim up to a length limit.
    pub(super) requests: Vec<String>,
    pub(super) members: Vec<MemberHandoff>,
    /// Latest shared records as `owner: content`, oldest first.
    pub(super) records: Vec<String>,
}

fn clip(text: &str, limit: usize) -> String {
    let text = text.trim();
    if text.chars().count() <= limit {
        return text.to_string();
    }
    text.chars().take(limit).collect::<String>() + " [shortened; full text in the history file]"
}

/// `artifact` content is a JSON array of paths; keep plain text as one entry.
fn artifact_paths(content: &str) -> Vec<String> {
    serde_json::from_str::<Vec<String>>(content)
        .unwrap_or_else(|_| vec![content.trim().to_string()])
        .into_iter()
        .filter(|path| !path.trim().is_empty())
        .collect()
}

impl HandoffBrief {
    pub(super) fn is_empty(&self) -> bool {
        self.requests.is_empty() && self.members.is_empty() && self.records.is_empty()
    }

    pub(super) fn render(&self) -> String {
        let mut out = String::new();
        if !self.requests.is_empty() {
            out.push_str("### Requests from the user\n");
            for request in &self.requests {
                out.push_str("- ");
                out.push_str(&request.replace('\n', "\n  "));
                out.push('\n');
            }
        }
        if !self.members.is_empty() {
            out.push_str("### Latest work by other members\n");
            for member in &self.members {
                out.push_str("- ");
                out.push_str(&member.name);
                out.push_str(": ");
                out.push_str(
                    member
                        .conclusion
                        .as_deref()
                        .unwrap_or("(no conclusion reported)"),
                );
                if !member.files.is_empty() {
                    out.push_str("\n  files: ");
                    out.push_str(&member.files.join(", "));
                }
                out.push('\n');
            }
        }
        if !self.records.is_empty() {
            out.push_str("### Shared records\n");
            for record in &self.records {
                out.push_str("- ");
                out.push_str(record);
                out.push('\n');
            }
        }
        out
    }

    /// Assemble a brief from session data. `members` maps session member ids
    /// to display names; `exclude` is the member being prompted.
    pub(super) fn from_session(
        messages: &[ChatMessage],
        source_message_id: Uuid,
        work_items: &[ChatWorkItem],
        records: &[(String, String)],
        members: &[(Uuid, String)],
        exclude: Uuid,
    ) -> Self {
        let mut requests = messages
            .iter()
            .rev()
            .filter(|message| {
                message.sender_type == ChatSenderType::User && message.id != source_message_id
            })
            .take(HANDOFF_USER_REQUESTS)
            .map(|message| clip(&message.content, HANDOFF_REQUEST_CHARS))
            .collect::<Vec<_>>();
        requests.reverse();

        let mut member_handoffs = Vec::new();
        for (member_id, name) in members.iter().filter(|(id, _)| *id != exclude) {
            let own = work_items
                .iter()
                .filter(|item| item.session_agent_id == *member_id)
                .collect::<Vec<_>>();
            let Some(latest_run) = own
                .iter()
                .max_by_key(|item| item.created_at)
                .map(|item| item.run_id)
            else {
                continue;
            };
            let latest = own.iter().filter(|item| item.run_id == latest_run);
            let mut handoff = MemberHandoff {
                name: name.clone(),
                conclusion: None,
                files: Vec::new(),
            };
            for item in latest {
                match item.item_type {
                    ChatWorkItemType::Conclusion => {
                        handoff.conclusion = Some(clip(&item.content, HANDOFF_CONCLUSION_CHARS));
                    }
                    ChatWorkItemType::Artifact => {
                        handoff.files.extend(artifact_paths(&item.content))
                    }
                }
            }
            handoff.files.sort();
            handoff.files.dedup();
            handoff.files.truncate(HANDOFF_MAX_FILES);
            member_handoffs.push(handoff);
        }

        let skip = records.len().saturating_sub(HANDOFF_RECORDS);
        let records = records[skip..]
            .iter()
            .map(|(owner, content)| format!("{owner}: {}", clip(content, HANDOFF_CONCLUSION_CHARS)))
            .collect();

        Self {
            requests,
            members: member_handoffs,
            records,
        }
    }
}

impl ChatRunner {
    /// Shared records written by members through the `record` protocol item.
    async fn read_shared_records(session_id: Uuid) -> Vec<(String, String)> {
        let path = Self::session_shared_blackboard_path(session_id);
        let Ok(text) = fs::read_to_string(&path).await else {
            return Vec::new();
        };
        text.lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .filter(|entry| entry.get("message_type").and_then(|v| v.as_str()) == Some("record"))
            .filter_map(|entry| {
                Some((
                    entry.get("owner")?.as_str()?.to_string(),
                    entry.get("content")?.as_str()?.to_string(),
                ))
            })
            .collect()
    }

    pub(super) async fn build_handoff_brief(
        &self,
        messages: &[ChatMessage],
        source_message: &ChatMessage,
        members: &[(Uuid, String)],
        exclude: Uuid,
    ) -> Option<String> {
        let work_items =
            ChatWorkItem::find_by_session_id(&self.db.pool, source_message.session_id, None)
                .await
                .unwrap_or_else(|err| {
                    tracing::warn!(error = %err, "handoff brief: failed to load work items");
                    Vec::new()
                });
        let records = Self::read_shared_records(source_message.session_id).await;
        let brief = HandoffBrief::from_session(
            messages,
            source_message.id,
            &work_items,
            &records,
            members,
            exclude,
        );
        (!brief.is_empty()).then(|| brief.render())
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;
    use serde_json::json;

    use super::*;

    fn message(sender_type: ChatSenderType, content: &str) -> ChatMessage {
        ChatMessage {
            client_op_key: None,
            id: Uuid::new_v4(),
            session_id: Uuid::nil(),
            sender_type,
            sender_id: None,
            sender_session_agent_id: None,
            content: content.to_string(),
            mentions: sqlx::types::Json(Vec::new()),
            meta: sqlx::types::Json(json!({})),
            created_at: Utc::now(),
        }
    }

    fn item(
        member: Uuid,
        run: Uuid,
        kind: ChatWorkItemType,
        content: &str,
        age_secs: i64,
    ) -> ChatWorkItem {
        ChatWorkItem {
            id: Uuid::new_v4(),
            session_id: Uuid::nil(),
            run_id: run,
            session_agent_id: member,
            agent_id: Uuid::new_v4(),
            item_type: kind,
            content: content.to_string(),
            created_at: Utc::now() - Duration::seconds(age_secs),
        }
    }

    #[test]
    fn brief_keeps_requests_latest_work_and_records_but_not_the_reader() {
        let (coder, reviewer) = (Uuid::new_v4(), Uuid::new_v4());
        let (old_run, new_run) = (Uuid::new_v4(), Uuid::new_v4());
        let first = message(ChatSenderType::User, "@coder move login to OAuth");
        let current = message(ChatSenderType::User, "@reviewer check the change");
        let messages = vec![
            first,
            message(ChatSenderType::Agent, "done"),
            current.clone(),
        ];
        let items = vec![
            item(
                coder,
                old_run,
                ChatWorkItemType::Conclusion,
                "first attempt",
                60,
            ),
            item(
                coder,
                new_run,
                ChatWorkItemType::Conclusion,
                "OAuth callback added; tests pass",
                5,
            ),
            item(
                coder,
                new_run,
                ChatWorkItemType::Artifact,
                r#"["src/auth.rs","src/config.rs"]"#,
                5,
            ),
            item(
                reviewer,
                Uuid::new_v4(),
                ChatWorkItemType::Conclusion,
                "earlier review",
                30,
            ),
        ];
        let records = vec![(
            "coder".to_string(),
            "Callback URL comes from config only".to_string(),
        )];
        let members = vec![
            (coder, "coder".to_string()),
            (reviewer, "reviewer".to_string()),
        ];

        let brief =
            HandoffBrief::from_session(&messages, current.id, &items, &records, &members, reviewer);

        assert_eq!(
            brief.requests,
            vec!["@coder move login to OAuth".to_string()]
        );
        assert_eq!(
            brief.members,
            vec![MemberHandoff {
                name: "coder".to_string(),
                conclusion: Some("OAuth callback added; tests pass".to_string()),
                files: vec!["src/auth.rs".to_string(), "src/config.rs".to_string()],
            }]
        );
        let text = brief.render();
        assert!(text.contains("coder: Callback URL comes from config only"));
        assert!(
            !text.contains("earlier review"),
            "the reader's own work is not handed to itself"
        );
        assert!(
            !text.contains("first attempt"),
            "only the latest run of each member is kept"
        );
    }

    #[test]
    fn empty_session_has_no_brief() {
        let current = message(ChatSenderType::User, "@coder start");
        let brief = HandoffBrief::from_session(
            &[current.clone()],
            current.id,
            &[],
            &[],
            &[],
            Uuid::new_v4(),
        );
        assert!(brief.is_empty());
    }
}
