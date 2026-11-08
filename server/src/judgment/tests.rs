use super::*;
use rusqlite::Connection;

fn fixture() -> (tempfile::TempDir, Database) {
    let dir = tempfile::tempdir().unwrap();
    let db = Database::new(dir.path().join("test.db")).unwrap();
    initialize(&db, None).unwrap();
    (dir, db)
}
fn query(v: Value) -> Query {
    serde_json::from_value(v).unwrap()
}
fn answer(case: &Value) -> Value {
    let count = case["view"]["min_cards"].as_u64().unwrap() as usize;
    json!({"case_id":case["id"],"expected_revision":case["revision"],"status":"answered",
        "action":case["view"]["legal_cards"].as_array().unwrap()[..count]})
}
fn submit(db: &Database, u: UserId, v: Value) -> Result<Value, CardsError> {
    review(db, u, serde_json::from_value(v).unwrap())
}

#[tokio::test(flavor = "multi_thread")]
async fn fresh_and_existing_database_migrate_without_losing_data() {
    for version in [0, 1] {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("migration.db");
        if version == 1 {
            let conn = Connection::open(&path).unwrap();
            conn.execute_batch(include_str!("../../sql/schema.sql"))
                .unwrap();
            conn.execute(
                "INSERT INTO user VALUES('fixture','Existing player','test','existing')",
                [],
            )
            .unwrap();
            conn.pragma_update(None, "user_version", 1).unwrap();
        }
        let db = Database::new(&path).unwrap();
        initialize(&db, None).unwrap();
        initialize(&db, None).unwrap(); // Restart/import is idempotent.
        let conn = Connection::open(&path).unwrap();
        assert_eq!(
            conn.query_row("PRAGMA user_version", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            2
        );
        assert_eq!(
            conn.query_row("SELECT count(*) FROM judgment_case", [], |r| r
                .get::<_, i64>(0))
                .unwrap(),
            240
        );
        assert_eq!(
            conn.query_row("SELECT count(*) FROM user", [], |r| r.get::<_, i64>(0))
                .unwrap(),
            version
        );
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn assignments_isolate_users_and_prevent_cross_game_hindsight() {
    let (_dir, db) = fixture();
    let alice = UserId::new();
    let bob = UserId::new();
    let first = state(&db, alice, Query::default()).unwrap();
    let c = &first["case"];
    assert!(c.get("context").is_none());
    assert!(c.get("selection").is_none());
    assert!(c.get("group_id").is_none());
    assert!(c.get("source_split").is_none());
    assert_eq!(first, state(&db, alice, Query::default()).unwrap());
    assert!(state(&db, bob, query(json!({"case":c["id"]}))).is_err());
    assert!(submit(&db, bob, answer(c)).is_err());
    submit(&db, alice, answer(c)).unwrap();
    let second = state(&db, alice, Query::default()).unwrap();
    assert_ne!(
        c["id"].as_str().unwrap().split(':').next(),
        second["case"]["id"].as_str().unwrap().split(':').next()
    );
    let edited = state(&db, alice, query(json!({"case":c["id"]}))).unwrap();
    assert_eq!(edited["case"]["revision"], 1);
    assert_eq!(
        edited["case"]["annotation"]["label_semantics"],
        "expert_recommendation"
    );
    let b = state(&db, bob, Query::default()).unwrap();
    assert_eq!(b["recent"], json!([]));
    assert_eq!(b["case"]["annotation"], Value::Null);
    // Alternate question IDs in an exposed game may not be fetched directly.
    let sibling: String = db.run_read_only(|tx| Ok(tx.query_row("SELECT b.id FROM judgment_case a JOIN judgment_case b ON a.group_id=b.group_id WHERE a.id=? AND b.id!=a.id LIMIT 1", [c["id"].as_str().unwrap()],|r| r.get(0))?)).unwrap();
    assert!(state(&db, alice, query(json!({"case":sibling}))).is_err());
}

#[tokio::test(flavor = "multi_thread")]
async fn validate_actions_revision_history_and_durable_resume() {
    let (dir, db) = fixture();
    let u = UserId::new();
    let state1 = state(&db, u, query(json!({"kind":"pass"}))).unwrap();
    let c = &state1["case"];
    let good = answer(c);
    for action in [
        json!([]),
        json!(["ZZ", "AS", "2C"]),
        json!(["AS", "AS", "AS"]),
    ] {
        let mut b = good.clone();
        b["action"] = action;
        assert!(submit(&db, u, b).is_err());
    }
    let mut b = good.clone();
    b["alternatives"] = json!([good["action"]]);
    assert!(submit(&db, u, b).is_err());
    let mut b = good.clone();
    b["suspected_runner"] = json!("other");
    assert!(submit(&db, u, b).is_err());
    let mut b = good.clone();
    b["note"] = json!("keep a long-suit stopper");
    b["recognized"] = json!(true);
    let legal = c["view"]["legal_cards"].as_array().unwrap();
    b["alternatives"] = json!([legal[1..4].to_vec()]);
    submit(&db, u, b).unwrap();
    assert!(matches!(
        submit(&db, u, good.clone()),
        Err(CardsError::Judgment { status: 409, .. })
    ));
    let mut skip = good;
    skip["expected_revision"] = json!(1);
    skip["status"] = json!("skipped");
    submit(&db, u, skip).unwrap();
    let db2 = Database::new(dir.path().join("test.db")).unwrap();
    let resumed = state(&db2, u, query(json!({"case":c["id"]}))).unwrap();
    assert_eq!(resumed["case"]["revision"], 2);
    assert_eq!(resumed["case"]["annotation"]["status"], "skipped");
    assert_eq!(resumed["case"]["annotation"]["action"], Value::Null);
    db2.run_read_only(|tx| {
        assert_eq!(
            tx.query_row("SELECT count(*) FROM judgment_response", [], |r| r
                .get::<_, i64>(0))?,
            2
        );
        Ok(())
    })
    .unwrap();
    let charge = state(&db, u, query(json!({"kind":"charge"}))).unwrap();
    let mut b = answer(&charge["case"]);
    b.as_object_mut().unwrap().remove("action");
    assert!(submit(&db, u, b).is_err());
    submit(&db, u, answer(&charge["case"])).unwrap(); // Explicit no charge.
    let play = state(&db, u, query(json!({"kind":"play"}))).unwrap();
    submit(&db, u, answer(&play["case"])).unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn import_failure_rolls_back_all_rows() {
    let (_dir, db) = fixture();
    let base: Value = serde_json::from_str(
        include_str!("../../judgment-cases.jsonl")
            .lines()
            .next()
            .unwrap(),
    )
    .unwrap();
    let mut new = base.clone();
    new["id"] = json!("new");
    let mut bad = base;
    bad["view"]["hand"] = json!(["ZZ"]);
    assert!(import(&db, &format!("{}\n{}", new, bad)).is_err());
    db.run_read_only(|tx| {
        assert_eq!(
            tx.query_row(
                "SELECT count(*) FROM judgment_case WHERE id='new'",
                [],
                |r| r.get::<_, i64>(0)
            )?,
            0
        );
        Ok(())
    })
    .unwrap();
}

#[tokio::test(flavor = "multi_thread")]
async fn concurrent_requests_resume_one_assignment_and_conflict_on_revision() {
    let (_dir, db) = fixture();
    let db = std::sync::Arc::new(db);
    let u = UserId::new();
    let mut tasks = Vec::new();
    for _ in 0..8 {
        let db = db.clone();
        tasks.push(tokio::spawn(async move {
            state(&db, u, Query::default()).unwrap()
        }));
    }
    let mut ids = HashSet::new();
    for t in tasks {
        ids.insert(t.await.unwrap()["case"]["id"].as_str().unwrap().to_owned());
    }
    assert_eq!(ids.len(), 1);
    let c = state(&db, u, Query::default()).unwrap();
    let mut tasks = Vec::new();
    for _ in 0..8 {
        let db = db.clone();
        let b = answer(&c["case"]);
        tasks.push(tokio::spawn(async move { submit(&db, u, b) }));
    }
    let mut saved = 0;
    let mut conflicts = 0;
    for t in tasks {
        match t.await.unwrap() {
            Ok(_) => saved += 1,
            Err(CardsError::Judgment { status: 409, .. }) => conflicts += 1,
            e => panic!("{:?}", e),
        }
    }
    assert_eq!((saved, conflicts), (1, 7));
}

#[test]
fn bundled_questions_have_only_observed_history() {
    let mut types = HashSet::new();
    for line in include_str!("../../judgment-cases.jsonl").lines() {
        let c: Value = serde_json::from_str(line).unwrap();
        let v = &c["view"];
        assert_eq!(c["source_split"], 0); // Never train on held-out games by default.
        types.insert(v["kind"].as_str().unwrap().to_owned());
        let actor = v["seat"].as_str().unwrap();
        for event in v["history"].as_array().unwrap() {
            let kind = event["type"].as_str().unwrap();
            assert!(!["sit", "chat", "hand_complete", "game_complete"].contains(&kind));
            assert!(event.get("user_id").is_none());
            assert!(event.get("seed").is_none());
            if kind == "deal" {
                for seat in ["north", "east", "south", "west"] {
                    if seat != actor {
                        assert_eq!(event[seat], json!([]));
                    }
                }
            }
            if kind == "send_pass" {
                assert_eq!(event["from"], actor);
            }
            if kind == "recv_pass" {
                assert_eq!(event["to"], actor);
            }
            if kind == "charge"
                && v["kind"] == "charge"
                && v["rules"].as_str().unwrap().contains("blind")
            {
                assert_eq!(event["seat"], actor);
            }
        }
    }
    assert_eq!(types.len(), 3);
}
