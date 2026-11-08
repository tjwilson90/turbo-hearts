use super::*;
use crate::{ExternalUser, Users};
use serde_json::Value;

#[tokio::test(flavor = "multi_thread")]
async fn authenticated_same_origin_api_and_private_projection() {
    let dir = tempfile::tempdir().unwrap();
    let db: &'static Database =
        Box::leak(Box::new(Database::new(dir.path().join("http.db")).unwrap()));
    judgment::initialize(db, None).unwrap();
    let users: &'static Users = Box::leak(Box::new(Users::new(db)));
    users
        .insert(
            "alice-cookie".into(),
            ExternalUser {
                name: "Alice".into(),
                realm: "fixture".into(),
                external_id: "alice".into(),
            },
        )
        .await
        .unwrap();
    users
        .insert(
            "bob-cookie".into(),
            ExternalUser {
                name: "Bob".into(),
                realm: "fixture".into(),
                external_id: "bob".into(),
            },
        )
        .await
        .unwrap();
    let app = router(
        warp::any().map(move || db),
        crate::user_id(warp::any().map(move || users)),
        "https://hearts.example",
    );
    let req = || {
        warp::test::request()
            .method("POST")
            .path("/judge/api/state")
            .header("origin", "https://hearts.example")
            .header("x-judgment-request", "1")
    };
    assert_eq!(req().json(&json!({})).reply(&app).await.status(), 401);
    assert_eq!(
        req()
            .header("cookie", "AUTH_TOKEN=invalid-secret")
            .json(&json!({}))
            .reply(&app)
            .await
            .status(),
        401
    );
    assert_eq!(
        req()
            .header("origin", "https://evil.example")
            .header("cookie", "AUTH_TOKEN=alice-cookie")
            .json(&json!({}))
            .reply(&app)
            .await
            .status(),
        403
    );
    assert_eq!(
        warp::test::request()
            .method("POST")
            .path("/judge/api/state")
            .header("origin", "https://hearts.example")
            .header("cookie", "AUTH_TOKEN=alice-cookie")
            .json(&json!({}))
            .reply(&app)
            .await
            .status(),
        403
    );
    let result = req()
        .header("cookie", "AUTH_TOKEN=alice-cookie")
        .json(&json!({}))
        .reply(&app)
        .await;
    assert_eq!(result.status(), 200);
    assert_eq!(result.headers()[header::CACHE_CONTROL], "no-store");
    let state: Value = serde_json::from_slice(result.body()).unwrap();
    let c = &state["case"];
    let text = String::from_utf8(result.body().to_vec()).unwrap();
    for hidden in [
        "policy_choices",
        "source_split",
        "event_index",
        "post_pass_hand",
        "user_id",
    ] {
        assert!(!text.contains(hidden), "{}", hidden);
    }
    assert_eq!(
        req()
            .header("cookie", "AUTH_TOKEN=bob-cookie")
            .json(&json!({"case":c["id"]}))
            .reply(&app)
            .await
            .status(),
        404
    );
    let mut answer = json!({"case_id":c["id"],"expected_revision":0,"status":"skipped"});
    let review = || {
        warp::test::request()
            .method("POST")
            .path("/judge/api/review")
            .header("origin", "https://hearts.example")
            .header("x-judgment-request", "1")
            .header("cookie", "AUTH_TOKEN=alice-cookie")
    };
    answer["reviewer"] = json!("Bob");
    assert_eq!(review().json(&answer).reply(&app).await.status(), 400);
    answer.as_object_mut().unwrap().remove("reviewer");
    assert_eq!(review().json(&answer).reply(&app).await.status(), 200);
    assert_eq!(review().json(&answer).reply(&app).await.status(), 409);
    let page = warp::test::request().path("/judge").reply(&app).await;
    assert_eq!(page.status(), 303);
    assert_eq!(page.headers()[header::LOCATION], "/auth");
    let page = warp::test::request()
        .path("/judge")
        .header("cookie", "AUTH_TOKEN=alice-cookie")
        .reply(&app)
        .await;
    assert_eq!(page.status(), 200);
    assert!(String::from_utf8(page.body().to_vec())
        .unwrap()
        .contains("/judge/app.js"));
    assert_eq!(
        warp::test::request()
            .path("/judge/api/state")
            .reply(&app)
            .await
            .status(),
        405
    );
    assert_eq!(
        warp::test::request()
            .path("/judge/cases.jsonl")
            .reply(&app)
            .await
            .status(),
        404
    );
}
