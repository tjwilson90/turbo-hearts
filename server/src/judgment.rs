//! Human recommendations are separate from observed moves and reward targets.
use crate::{CardsError, Database};
use rusqlite::{params, DropBehavior, OptionalExtension, Transaction};
use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashSet;
use turbo_hearts_api::UserId;

const KINDS: &[&str] = &["pass", "charge", "play"];
const DIRECTIONS: &[&str] = &["left", "right", "across", "keeper"];
const INTENTS: &[&str] = &[
    "avoid_points",
    "shoot_moon",
    "stop_moon",
    "win_jack",
    "manage_ten",
    "create_void",
    "keep_control",
    "shed_low_cards",
    "give_points",
    "other",
];
pub(crate) fn error(status: u16, message: &str) -> CardsError {
    CardsError::Judgment {
        status,
        message: message.into(),
    }
}
fn invalid(message: &str) -> CardsError {
    error(400, message)
}

pub fn initialize(db: &Database, extra_path: Option<&str>) -> Result<(), CardsError> {
    import(db, include_str!("../judgment-cases.jsonl"))?;
    if let Some(path) = extra_path {
        let text = std::fs::read_to_string(path)
            .map_err(|e| CardsError::Configuration(format!("judgment_cases_path: {}", e)))?;
        import(db, &text)?;
    }
    Ok(())
}

fn card(c: &str) -> bool {
    c.len() == 2
        && b"23456789TJQKA".contains(&c.as_bytes()[0])
        && b"CDHS".contains(&c.as_bytes()[1])
}
fn strings(value: &Value) -> Result<Vec<String>, CardsError> {
    serde_json::from_value(value.clone()).map_err(|_| invalid("Expected a list of cards."))
}
fn canonical(cards: &[String], view: &Value) -> Result<Vec<String>, CardsError> {
    let legal = strings(&view["legal_cards"])?;
    let min = view["min_cards"]
        .as_u64()
        .ok_or_else(|| invalid("Invalid question size."))? as usize;
    let max = view["max_cards"]
        .as_u64()
        .ok_or_else(|| invalid("Invalid question size."))? as usize;
    let mut result = cards.to_vec();
    result.sort();
    result.dedup();
    if result.len() != cards.len()
        || cards.len() < min
        || cards.len() > max
        || cards.iter().any(|c| !legal.contains(c))
    {
        return Err(invalid(
            "Choose the required number of distinct, available cards.",
        ));
    }
    Ok(result)
}

pub(crate) fn import(db: &Database, text: &str) -> Result<usize, CardsError> {
    let cases: Vec<Value> = text
        .lines()
        .filter(|s| !s.trim().is_empty())
        .map(serde_json::from_str)
        .collect::<Result<_, _>>()?;
    db.run_with_retry(|mut tx| {
        tx.set_drop_behavior(DropBehavior::Rollback);
        let mut order: i64 = tx.query_row(
            "SELECT coalesce(max(queue_order),-1)+1 FROM judgment_case",
            [],
            |r| r.get(0),
        )?;
        let mut added = 0;
        for c in &cases {
            let v = &c["view"];
            let id = c["id"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| invalid("Missing question ID."))?;
            let group = c["group_id"]
                .as_str()
                .filter(|s| !s.is_empty())
                .ok_or_else(|| invalid("Missing question group."))?;
            let kind = v["kind"].as_str().unwrap_or("");
            let direction = v["direction"].as_str().unwrap_or("");
            let hand = strings(&v["hand"])?;
            let legal = strings(&v["legal_cards"])?;
            if c["schema"] != "turbo-hearts-judgment-v1"
                || !KINDS.contains(&kind)
                || !DIRECTIONS.contains(&direction)
                || hand.iter().any(|s| !card(s))
                || hand.iter().collect::<HashSet<_>>().len() != hand.len()
                || legal.iter().any(|s| !hand.contains(s))
                || legal.iter().collect::<HashSet<_>>().len() != legal.len()
                || legal.is_empty()
                || c["context"].is_null()
                || !matches!(c["source_split"].as_u64(), Some(0..=2))
            {
                return Err(invalid("Invalid judgment question."));
            }
            let (min, max) = match kind {
                "pass" => (3, 3),
                "play" => (1, 1),
                _ => (0, legal.len()),
            };
            if v["min_cards"] != json!(min) || v["max_cards"] != json!(max) {
                return Err(invalid("Invalid question action limits."));
            }
            let payload = serde_json::to_string(c)?;
            let previous: Option<String> = tx
                .query_row("SELECT payload FROM judgment_case WHERE id=?", [id], |r| {
                    r.get(0)
                })
                .optional()?;
            if let Some(previous) = previous {
                if serde_json::from_str::<Value>(&previous)? != *c {
                    return Err(invalid(
                        "An existing question changed. Use a new version/ID.",
                    ));
                }
                continue;
            }
            tx.execute(
                "INSERT INTO judgment_case VALUES(?,?,?,?,?,?)",
                params![id, group, kind, direction, payload, order],
            )?;
            order += 1;
            added += 1;
        }
        tx.commit()?;
        Ok(added)
    })
}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Query {
    kind: Option<String>,
    direction: Option<String>,
    case: Option<String>,
}
fn latest(tx: &Transaction, user: &str, id: &str) -> Result<(i64, Option<Value>), CardsError> {
    let row: Option<(i64, String)> = tx.query_row(
        "SELECT revision,payload FROM judgment_response WHERE user_id=? AND case_id=? ORDER BY revision DESC LIMIT 1",
        params![user,id], |r| Ok((r.get(0)?,r.get(1)?))).optional()?;
    match row {
        Some((rev, payload)) => Ok((rev, Some(serde_json::from_str(&payload)?))),
        None => Ok((0, None)),
    }
}

pub fn state(db: &Database, user: UserId, q: Query) -> Result<Value, CardsError> {
    let user = user.to_string();
    let kind = q.kind.as_deref().unwrap_or("all");
    let direction = q.direction.as_deref().unwrap_or("all");
    if (kind != "all" && !KINDS.contains(&kind))
        || (direction != "all" && !DIRECTIONS.contains(&direction))
    {
        return Err(invalid("Unknown decision or round."));
    }
    db.run_with_retry(|mut tx| {
        tx.set_drop_behavior(DropBehavior::Rollback);
        let id: Option<String> = if let Some(id) = &q.case {
            // Explicit IDs may only resume already exposed positions.
            if !tx.query_row("SELECT EXISTS(SELECT 1 FROM judgment_assignment WHERE user_id=? AND case_id=?)", params![user,id], |r| r.get::<_,bool>(0))? {
                return Err(error(404,"Question not assigned to your account."));
            }
            Some(id.clone())
        } else {
            let pending = tx.query_row("SELECT c.id FROM judgment_case c JOIN judgment_assignment a ON a.case_id=c.id
                WHERE a.user_id=?1 AND (?2='all' OR c.kind=?2) AND (?3='all' OR c.direction=?3)
                AND NOT EXISTS(SELECT 1 FROM judgment_response r WHERE r.user_id=?1 AND r.case_id=c.id)
                ORDER BY a.created_at,c.queue_order LIMIT 1", params![user,kind,direction], |r| r.get::<_,String>(0)).optional()?;
            if pending.is_some() { pending } else {
                // Balance task/round exposure within each account, then spread independent reviews.
                let next: Option<(String,String)> = tx.query_row("SELECT c.id,c.group_id FROM judgment_case c
                    WHERE (?2='all' OR c.kind=?2) AND (?3='all' OR c.direction=?3)
                    AND NOT EXISTS(SELECT 1 FROM judgment_assignment a WHERE a.user_id=?1 AND a.group_id=c.group_id)
                    ORDER BY (SELECT count(*) FROM judgment_assignment a JOIN judgment_case b ON b.id=a.case_id
                        WHERE a.user_id=?1 AND b.kind=c.kind),
                        (SELECT count(*) FROM judgment_assignment a JOIN judgment_case b ON b.id=a.case_id
                        WHERE a.user_id=?1 AND b.kind=c.kind AND b.direction=c.direction),
                        (SELECT count(*) FROM judgment_assignment a WHERE a.case_id=c.id), c.queue_order,c.id LIMIT 1",
                    params![user,kind,direction], |r| Ok((r.get(0)?,r.get(1)?))).optional()?;
                if let Some((id,group)) = next {
                    tx.execute("INSERT INTO judgment_assignment(user_id,case_id,group_id) VALUES(?,?,?)", params![user,id,group])?;
                    Some(id)
                } else { None }
            }
        };
        let case = if let Some(id) = id {
            let payload: String = tx.query_row("SELECT payload FROM judgment_case WHERE id=?", [&id], |r| r.get(0))?;
            let c: Value = serde_json::from_str(&payload)?;
            let (revision, annotation) = latest(&tx,&user,&id)?;
            // Explicit allowlist: never send internal state, source IDs, policy suggestions or other labels.
            json!({"id":id,"view":c["view"],"revision":revision,"annotation":annotation})
        } else { Value::Null };
        let mut recent = Vec::new();
        {
            let mut stmt = tx.prepare("SELECT r.case_id,c.kind,c.direction,r.status,r.created_at FROM judgment_response r
                JOIN judgment_case c ON c.id=r.case_id WHERE r.user_id=? AND r.revision=(SELECT max(rr.revision)
                FROM judgment_response rr WHERE rr.user_id=r.user_id AND rr.case_id=r.case_id) ORDER BY r.id DESC LIMIT 30")?;
            for row in stmt.query_map([&user], |r| Ok(json!({"case_id":r.get::<_,String>(0)?,"kind":r.get::<_,String>(1)?,
                "direction":r.get::<_,String>(2)?,"status":r.get::<_,String>(3)?,"created_at":r.get::<_,String>(4)?})))? {
                recent.push(row?);
            }
        }
        let answered: i64 = tx.query_row("SELECT count(*) FROM judgment_response r WHERE user_id=? AND status='answered'
            AND revision=(SELECT max(rr.revision) FROM judgment_response rr WHERE rr.user_id=r.user_id AND rr.case_id=r.case_id)", [&user], |r| r.get(0))?;
        let skipped: i64 = tx.query_row("SELECT count(*) FROM judgment_response r WHERE user_id=? AND status='skipped'
            AND revision=(SELECT max(rr.revision) FROM judgment_response rr WHERE rr.user_id=r.user_id AND rr.case_id=r.case_id)", [&user], |r| r.get(0))?;
        let pending: i64 = tx.query_row("SELECT count(DISTINCT group_id) FROM judgment_case c
            WHERE NOT EXISTS(SELECT 1 FROM judgment_assignment a WHERE a.user_id=?1 AND a.group_id=c.group_id)
            OR EXISTS(SELECT 1 FROM judgment_assignment a WHERE a.user_id=?1 AND a.case_id=c.id AND NOT EXISTS
                (SELECT 1 FROM judgment_response r WHERE r.user_id=?1 AND r.case_id=c.id))", [&user], |r| r.get(0))?;
        tx.commit()?;
        Ok(json!({"case":case,"progress":{"all":{"answered":answered,"skipped":skipped,"pending":pending}},"recent":recent}))
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Review {
    case_id: String,
    expected_revision: i64,
    status: String,
    action: Option<Vec<String>>,
    #[serde(default)]
    alternatives: Vec<Vec<String>>,
    #[serde(default)]
    confidence: Option<String>,
    #[serde(default)]
    intents: Vec<String>,
    #[serde(default)]
    suspected_runner: Option<String>,
    #[serde(default)]
    note: String,
    #[serde(default)]
    recognized: bool,
    #[serde(default)]
    viewing_seconds: f64,
}

pub fn review(db: &Database, user: UserId, body: Review) -> Result<Value, CardsError> {
    let user = user.to_string();
    let confidence = body.confidence.as_deref().filter(|s| !s.is_empty());
    let runner = body.suspected_runner.as_deref().unwrap_or("unknown");
    if body.expected_revision < 0
        || !["answered", "skipped"].contains(&body.status.as_str())
        || body.alternatives.len() > 20
        || body.intents.len() > INTENTS.len()
        || body.intents.iter().any(|s| !INTENTS.contains(&s.as_str()))
        || confidence.is_some_and(|s| !["high", "medium", "low"].contains(&s))
        || !["north", "east", "south", "west", "unknown", "none"].contains(&runner)
        || body.note.chars().count() > 4000
        || !body.viewing_seconds.is_finite()
        || !(0.0..=86400.0).contains(&body.viewing_seconds)
    {
        return Err(invalid("Invalid judgment metadata."));
    }
    db.run_with_retry(|mut tx| {
        tx.set_drop_behavior(DropBehavior::Rollback);
        let payload: String = tx.query_row("SELECT c.payload FROM judgment_case c JOIN judgment_assignment a ON a.case_id=c.id
            WHERE a.user_id=? AND a.case_id=?", params![user,body.case_id], |r| r.get(0)).optional()?
            .ok_or_else(|| error(404,"Question not assigned to your account."))?;
        let (rev,_) = latest(&tx,&user,&body.case_id)?;
        if body.expected_revision != rev { return Err(error(409,"This judgment changed in another tab. Reload before editing.")); }
        let c: Value = serde_json::from_str(&payload)?;
        let mut alternatives = Vec::new();
        let action = if body.status == "answered" {
            let action = canonical(body.action.as_ref().ok_or_else(|| invalid("Choose an action, including an explicit no-charge choice."))?, &c["view"])?;
            for a in &body.alternatives {
                let a = canonical(a,&c["view"])?;
                if a == action || alternatives.contains(&a) { return Err(invalid("Alternative choices must be distinct.")); }
                alternatives.push(a);
            }
            Some(action)
        } else { None };
        let mut intents = body.intents.clone(); intents.sort(); intents.dedup();
        let label = json!({"status":body.status,"action":action,"alternatives":alternatives,
            "confidence":confidence,"intents":intents,"suspected_runner":runner,"note":body.note.trim(),
            "recognized":body.recognized,"viewing_seconds":body.viewing_seconds,"label_semantics":"expert_recommendation"});
        tx.execute("INSERT INTO judgment_response(case_id,user_id,revision,status,payload) VALUES(?,?,?,?,?)",
            params![body.case_id,user,rev+1,body.status,serde_json::to_string(&label)?])?;
        tx.commit()?;
        Ok(json!({"saved":true,"revision":rev+1}))
    })
}

#[cfg(test)]
mod tests;
