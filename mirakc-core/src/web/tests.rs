// <coverage:exclude>
use super::*;

use std::future::Future;

use assert_matches::assert_matches;
use axum::http::header::ACCEPT_RANGES;
use axum::http::header::LOCATION;
use axum::http::StatusCode;
use axum_test_helper::TestClient;
use axum_test_helper::TestResponse;

use crate::epg::stub::EpgStub;
use crate::onair::stub::OnairProgramManagerStub;
use crate::recording::stub::RecordingManagerStub;
use crate::recording::RecordingOptions;
use crate::timeshift::stub::TimeshiftManagerStub;
use crate::tuner::stub::TunerManagerStub;
use api::models::*;
use qs::*;

#[tokio::test]
async fn test_get_unknown() {
    let res = get("/api/unknown").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_version() {
    let res = get("/api/version").await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_status() {
    let res = get("/api/status").await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_channels() {
    let res = get("/api/channels").await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_services() {
    let res = get("/api/services").await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_service() {
    let res = get("/api/services/1").await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = get("/api/services/0").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_service_logo() {
    let res = get("/api/services/1/logo").await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = get("/api/services/0/logo").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let res = get("/api/services/2/logo").await;
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn test_head_service_logo() {
    let res = head("/api/services/1/logo").await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = head("/api/services/0/logo").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let res = head("/api/services/2/logo").await;
    assert_eq!(res.status(), StatusCode::SERVICE_UNAVAILABLE);
}

#[tokio::test]
async fn test_get_service_programs() {
    let res = get("/api/services/1/programs").await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = head("/api/services/0/programs").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_programs() {
    let res = get("/api/programs").await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_program() {
    let res = get("/api/programs/1").await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = get("/api/programs/0").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_tuners() {
    let res = get("/api/tuners").await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_tuner() {
    let res = get("/api/tuners/1").await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = get("/api/tuners/0").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_channel_stream() {
    let res = get("/api/channels/GR/ch/stream").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().contains_key(X_MIRAKURUN_TUNER_USER_ID));
    assert_matches!(res.headers().get(ACCEPT_RANGES), Some(v) => {
        assert_eq!(v, "none");
    });
    assert_matches!(res.headers().get("content-type"), Some(v) => {
        assert_eq!(v, "video/MP2T");
    });

    let res = get("/api/channels/GR/0/stream").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let res = get("/api/channels/WOWOW/ch/stream").await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let res = get("/api/channels/GR/xx/stream").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let decode_values = [0, 1];

    let valid_pairs = ["GR", "BS", "CS", "SKY"].iter().zip(&decode_values);
    for (channel_type, decode) in valid_pairs {
        let res =
            get(format!("/api/channels/{}/ch/stream?decode={}", channel_type, decode).as_str())
                .await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_matches!(res.headers().get(ACCEPT_RANGES), Some(v) => {
            assert_eq!(v, "none");
        });
    }

    for decode in &decode_values {
        let res = get(format!("/api/channels/WOWOW/ch/stream?decode={}", decode).as_str()).await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    let res = get("/api/channels/GR/ch/stream?post-filters[]=mp4").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_matches!(res.headers().get(ACCEPT_RANGES), Some(v) => {
        assert_eq!(v, "none");
    });
    assert_matches!(res.headers().get("content-type"), Some(v) => {
        assert_eq!(v, "video/mp4");
    });
}

#[tokio::test]
async fn test_get_channel_service_stream() {
    let res = get("/api/channels/GR/ch/services/1/stream").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().contains_key(X_MIRAKURUN_TUNER_USER_ID));
    assert_matches!(res.headers().get("content-type"), Some(v) => {
        assert_eq!(v, "video/MP2T");
    });
    assert_matches!(res.headers().get(ACCEPT_RANGES), Some(v) => {
        assert_eq!(v, "none");
    });

    let res = get("/api/channels/GR/0/services/1/stream").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let res = get("/api/channels/WOWOW/ch/services/1/stream").await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    let decode_values = [0, 1];

    let valid_pairs = ["GR", "BS", "CS", "SKY"].iter().zip(&decode_values);
    for (channel_type, decode) in valid_pairs {
        let res = get(format!(
            "/api/channels/{}/ch/services/1/stream?decode={}",
            channel_type, decode
        )
        .as_str())
        .await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_matches!(res.headers().get(ACCEPT_RANGES), Some(v) => {
            assert_eq!(v, "none");
        });
    }

    for decode in &decode_values {
        let res =
            get(format!("/api/channels/WOWOW/ch/services/1/stream?decode={}", decode).as_str())
                .await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    for decode in &decode_values {
        let res =
            get(format!("/api/channels/WOWOW/ch/services/2/stream?decode={}", decode).as_str())
                .await;
        assert_eq!(res.status(), StatusCode::BAD_REQUEST);
    }

    let res = get("/api/channels/GR/ch/services/1/stream?post-filters[]=mp4").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_matches!(res.headers().get(ACCEPT_RANGES), Some(v) => {
        assert_eq!(v, "none");
    });
    assert_matches!(res.headers().get("content-type"), Some(v) => {
        assert_eq!(v, "video/mp4");
    });
}

#[tokio::test]
async fn test_get_service_stream() {
    let res = get("/api/services/1/stream").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().contains_key(X_MIRAKURUN_TUNER_USER_ID));
    assert_matches!(res.headers().get("content-type"), Some(v) => {
        assert_eq!(v, "video/MP2T");
    });
    assert_matches!(res.headers().get(ACCEPT_RANGES), Some(v) => {
        assert_eq!(v, "none");
    });

    let res = get("/api/services/0/stream").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let decode_values = [0, 1];

    for decode in &decode_values {
        let res = get(format!("/api/services/1/stream?decode={}", decode).as_str()).await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_matches!(res.headers().get(ACCEPT_RANGES), Some(v) => {
                assert_eq!(v, "none");
        });
    }

    for decode in &decode_values {
        let res = get(format!("/api/services/0/stream?decode={}", decode).as_str()).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    for decode in &decode_values {
        let res = get(format!("/api/services/2/stream?decode={}", decode).as_str()).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    let res = get("/api/services/1/stream?post-filters[]=mp4").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_matches!(res.headers().get("content-type"), Some(v) => {
        assert_eq!(v, "video/mp4");
    });
    assert_matches!(res.headers().get(ACCEPT_RANGES), Some(v) => {
        assert_eq!(v, "none");
    });
}

#[tokio::test]
async fn test_head_service_stream() {
    let res = head("/api/services/1/stream").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_matches!(res.headers().get(ACCEPT_RANGES), Some(v) => {
        assert_eq!(v, "none");
    });
    assert_matches!(res.headers().get("content-type"), Some(v) => {
        assert_eq!(v, "video/MP2T");
    });

    let res = head("/api/services/0/stream").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // See comments in head_service_stream().
    let res = head("/api/services/2/stream").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_matches!(res.headers().get(ACCEPT_RANGES), Some(v) => {
        assert_eq!(v, "none");
    });

    let res = head("/api/services/1/stream?post-filters[]=mp4").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_matches!(res.headers().get(ACCEPT_RANGES), Some(v) => {
        assert_eq!(v, "none");
    });
    assert_matches!(res.headers().get("content-type"), Some(v) => {
        assert_eq!(v, "video/mp4");
    });
}

#[tokio::test]
async fn test_get_program_stream() {
    let res = get("/api/programs/100001/stream").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_matches!(res.headers().get(ACCEPT_RANGES), Some(v) => {
        assert_eq!(v, "none");
    });
    assert!(res.headers().contains_key(X_MIRAKURUN_TUNER_USER_ID));
    assert_matches!(res.headers().get("content-type"), Some(v) => {
        assert_eq!(v, "video/MP2T");
    });

    let res = get("/api/programs/0/stream").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let decode_values = [0, 1];

    for decode in &decode_values {
        let res = get(format!("/api/programs/100001/stream?decode={}", decode).as_str()).await;
        assert_eq!(res.status(), StatusCode::OK);
        assert_matches!(res.headers().get(ACCEPT_RANGES), Some(v) => {
            assert_eq!(v, "none");
        });
    }

    for decode in &decode_values {
        let res = get(format!("/api/programs/0/stream?decode={}", decode).as_str()).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    for decode in &decode_values {
        let res = get(format!("/api/programs/200001/stream?decode={}", decode).as_str()).await;
        assert_eq!(res.status(), StatusCode::NOT_FOUND);
    }

    let res = get("/api/programs/100001/stream?post-filters[]=mp4").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_matches!(res.headers().get(ACCEPT_RANGES), Some(v) => {
        assert_eq!(v, "none");
    });
    assert_matches!(res.headers().get("content-type"), Some(v) => {
        assert_eq!(v, "video/mp4");
    });
}

#[tokio::test]
async fn test_get_recording_schedules() {
    let res = get("/api/recording/schedules").await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_recording_schedule() {
    // Error::ProgramNotFound
    let input = WebRecordingScheduleInput {
        program_id: (0, 0, 0).into(),
        options: RecordingOptions {
            content_path: "0.m2ts".into(),
            priority: 1,
            pre_filters: vec![],
            post_filters: vec![],
        },
        tags: Default::default(),
    };
    let res = post("/api/recording/schedules", input).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    // Error::AlreadyExists
    let input = WebRecordingScheduleInput {
        program_id: (0, 0, 1).into(),
        options: RecordingOptions {
            content_path: "1.m2ts".into(),
            priority: 1,
            pre_filters: vec![],
            post_filters: vec![],
        },
        tags: Default::default(),
    };
    let res = post("/api/recording/schedules", input).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // Error::ProgramAlreadyStarted
    let input = WebRecordingScheduleInput {
        program_id: (0, 0, 2).into(),
        options: RecordingOptions {
            content_path: "2.m2ts".into(),
            priority: 1,
            pre_filters: vec![],
            post_filters: vec![],
        },
        tags: Default::default(),
    };
    let res = post("/api/recording/schedules", input).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // Ok
    let input = WebRecordingScheduleInput {
        program_id: (0, 0, 4).into(),
        options: RecordingOptions {
            content_path: "4.m2ts".into(),
            priority: 1,
            pre_filters: vec![],
            post_filters: vec![],
        },
        tags: Default::default(),
    };
    let res = post("/api/recording/schedules", input).await;
    assert_eq!(res.status(), StatusCode::CREATED);

    // Error::InvalidPath
    let input = WebRecordingScheduleInput {
        program_id: (0, 0, 4).into(),
        options: RecordingOptions {
            content_path: "/4.m2ts".into(),
            priority: 1,
            pre_filters: vec![],
            post_filters: vec![],
        },
        tags: Default::default(),
    };
    let res = post("/api/recording/schedules", input).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);

    // Error::InvalidPath
    let input = WebRecordingScheduleInput {
        program_id: (0, 0, 4).into(),
        options: RecordingOptions {
            content_path: "../4.m2ts".into(),
            priority: 1,
            pre_filters: vec![],
            post_filters: vec![],
        },
        tags: Default::default(),
    };
    let res = post("/api/recording/schedules", input).await;
    assert_eq!(res.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_get_recording_schedule() {
    let res = get("/api/recording/schedules/1").await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = get("/api/recording/schedules/0").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_recording_schedule() {
    let res = delete("/api/recording/schedules/1").await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = delete("/api/recording/schedules/0").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_recording_schedules() {
    let res = delete("/api/recording/schedules").await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = delete("/api/recording/schedules?target=tag").await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_recording_recorders() {
    let res = get("/api/recording/recorders").await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_create_recording_recorder() {
    let input = WebRecordingScheduleInput {
        program_id: (0, 0, 1).into(),
        options: RecordingOptions {
            content_path: "program.m2ts".into(),
            priority: 1,
            pre_filters: vec![],
            post_filters: vec![],
        },
        tags: Default::default(),
    };
    let res = post("/api/recording/recorders", input).await;
    assert_eq!(res.status(), StatusCode::CREATED);

    let input = WebRecordingScheduleInput {
        program_id: (0, 0, 0).into(),
        options: RecordingOptions {
            content_path: "program.m2ts".into(),
            priority: 1,
            pre_filters: vec![],
            post_filters: vec![],
        },
        tags: Default::default(),
    };
    let res = post("/api/recording/recorders", input).await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_recording_recorder() {
    let res = get("/api/recording/recorders/1").await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = get("/api/recording/recorders/0").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_delete_recording_recorder() {
    let res = delete("/api/recording/recorders/1").await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = delete("/api/recording/recorders/0").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_timeshift_recorders() {
    let res = get("/api/timeshift").await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_timeshift_recorder() {
    let res = get("/api/timeshift/test").await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = get("/api/timeshift/not_found").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_timeshift_records() {
    let res = get("/api/timeshift/test/records").await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_timeshift_record() {
    let res = get("/api/timeshift/test/records/1").await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = get("/api/timeshift/test/records/2").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_timeshift_stream() {
    let res = get("/api/timeshift/test/stream").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_matches!(res.headers().get(ACCEPT_RANGES), Some(v) => {
        assert_eq!(v, "none");
    });
    assert!(res.headers().contains_key(X_MIRAKURUN_TUNER_USER_ID));

    let res = get("/api/timeshift/not_found/stream").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_timeshift_record_stream() {
    let res = get("/api/timeshift/test/records/1/stream").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().contains_key(X_MIRAKURUN_TUNER_USER_ID));
    assert_matches!(res.headers().get(ACCEPT_RANGES), Some(v) => {
        assert_eq!(v, "bytes");
    });
    assert!(res.headers().contains_key("content-range"));
    assert!(res.headers().contains_key("content-length"));

    let res = get("/api/timeshift/test/records/1/stream?pre-filters[]=cat").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().contains_key(X_MIRAKURUN_TUNER_USER_ID));
    assert_matches!(res.headers().get(ACCEPT_RANGES), Some(v) => {
        assert_eq!(v, "bytes");
    });
    assert!(res.headers().contains_key("content-range"));
    assert!(res.headers().contains_key("content-length"));

    let res = get("/api/timeshift/test/records/1/stream?post-filters[]=cat").await;
    assert_eq!(res.status(), StatusCode::OK);
    assert!(res.headers().contains_key(X_MIRAKURUN_TUNER_USER_ID));
    assert_matches!(res.headers().get(ACCEPT_RANGES), Some(v) => {
        assert_eq!(v, "none");
    });
    assert!(!res.headers().contains_key("content-range"));
    assert!(!res.headers().contains_key("content-length"));

    let res = get("/api/timeshift/not_found/records/1/stream").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_iptv_playlist() {
    test_get_iptv_playlist_("/api/iptv/playlist").await;
}

#[tokio::test]
async fn test_get_iptv_channel_m3u8() {
    test_get_iptv_playlist_("/api/iptv/channel.m3u8").await;
}

async fn test_get_iptv_playlist_(endpoint: &str) {
    let res = get(endpoint).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_matches!(res.headers().get("content-type"), Some(v) => {
        assert_eq!(v, "application/x-mpegurl; charset=UTF-8");
    });
    let playlist = res.text().await;
    assert!(playlist.contains("#KODIPROP:mimetype=video/mp2t\n"));

    let res = get(&format!("{}?post-filters[]=mp4", endpoint)).await;
    assert_eq!(res.status(), StatusCode::OK);
    assert_matches!(res.headers().get("content-type"), Some(v) => {
        assert_eq!(v, "application/x-mpegurl; charset=UTF-8");
    });
    let playlist = res.text().await;
    assert!(playlist.contains("#KODIPROP:mimetype=video/mp4\n"));
}

#[tokio::test]
async fn test_get_iptv_epg() {
    let res = get("/api/iptv/epg").await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_iptv_xmltv() {
    let res = get("/api/iptv/xmltv").await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_list_onair() {
    let res = get("/api/onair").await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_onair() {
    let res = get("/api/onair/1").await;
    assert_eq!(res.status(), StatusCode::OK);

    let res = get("/api/onair/0").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_get_docs() {
    let res = get("/api/docs").await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_get_events() {
    let res = get("/events").await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_access_control_localhost() {
    let addr = "127.0.0.1:10000".parse().unwrap();
    let res = get_with_peer_addr("/api/version", Some(addr)).await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_access_control_uds() {
    let res = get_with_peer_addr("/api/version", None).await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_access_control_denied() {
    let addr = "8.8.8.8:10000".parse().unwrap();
    let res = get_with_peer_addr("/api/version", Some(addr)).await;
    assert_eq!(res.status(), StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_mount() {
    let res = get("/").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let res = get("/src").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);
    let res = get("/src/Cargo.toml").await;
    assert_eq!(res.status(), StatusCode::OK);
    let res = get("/src/no-such-file").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let res = get("/src-with-index").await;
    assert_eq!(res.status(), StatusCode::SEE_OTHER);
    assert_matches!(res.headers().get(LOCATION), Some(v) => {
        assert_eq!(v, "/src-with-index/Cargo.toml");
    });
    let res = get("/src-with-index/Cargo.toml").await;
    assert_eq!(res.status(), StatusCode::OK);
    let res = get("/src-with-index/no-such-file").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let res = get("/src-with-listing").await;
    assert_eq!(res.status(), StatusCode::OK);
    let res = get("/src-with-listing/Cargo.toml").await;
    assert_eq!(res.status(), StatusCode::OK);
    let res = get("/src-with-listing/no-such-file").await;
    assert_eq!(res.status(), StatusCode::NOT_FOUND);

    let res = get("/Cargo.toml").await;
    assert_eq!(res.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_filter_setting() {
    async fn do_test<H, F>(query: &str, handler: H) -> StatusCode
    where
        H: FnOnce(Qs<FilterSetting>) -> F + Clone + Send + 'static,
        F: Future<Output = ()> + Send,
    {
        let url = format!("/?{}", query);
        let app = Router::new().route("/", routing::get(handler));
        TestClient::new(app).get(&url).send().await.status()
    }

    assert_eq!(
        do_test("", |Qs(v)| async move {
            assert!(v.decode);
            assert!(v.pre_filters.is_empty());
            assert!(v.post_filters.is_empty());
        })
        .await,
        StatusCode::OK
    );

    assert_eq!(
        do_test("unknown=0", |Qs(v)| async move {
            assert!(v.decode);
            assert!(v.pre_filters.is_empty());
            assert!(v.post_filters.is_empty());
        })
        .await,
        StatusCode::OK
    );

    assert_eq!(
        do_test("decode=0", |Qs(v)| async move {
            assert!(!v.decode);
            assert!(v.pre_filters.is_empty());
            assert!(v.post_filters.is_empty());
        })
        .await,
        StatusCode::OK
    );

    assert_eq!(
        do_test("decode=1", |Qs(v)| async move {
            assert!(v.decode);
            assert!(v.pre_filters.is_empty());
            assert!(v.post_filters.is_empty());
        })
        .await,
        StatusCode::OK
    );

    assert_eq!(
        do_test("decode=false", |Qs(v)| async move {
            assert!(!v.decode);
            assert!(v.pre_filters.is_empty());
            assert!(v.post_filters.is_empty());
        })
        .await,
        StatusCode::OK
    );

    assert_eq!(
        do_test("decode=true", |Qs(v)| async move {
            assert!(v.decode);
            assert!(v.pre_filters.is_empty());
            assert!(v.post_filters.is_empty());
        })
        .await,
        StatusCode::OK
    );

    assert_eq!(
        do_test("decode=x", |_| async move {}).await,
        StatusCode::BAD_REQUEST
    );

    assert_eq!(
        do_test("pre-filters[]=a", |Qs(v)| async move {
            assert!(v.decode);
            assert_eq!(v.pre_filters.len(), 1);
            assert_eq!(v.pre_filters[0], "a".to_string());
            assert!(v.post_filters.is_empty());
        })
        .await,
        StatusCode::OK
    );

    assert_eq!(
        do_test("pre-filters[]=a&pre-filters[]=b", |Qs(v)| async move {
            assert!(v.decode);
            assert_eq!(v.pre_filters.len(), 2);
            assert_eq!(v.pre_filters[0], "a".to_string());
            assert_eq!(v.pre_filters[1], "b".to_string());
            assert!(v.post_filters.is_empty());
        })
        .await,
        StatusCode::OK
    );

    assert_eq!(
        do_test("pre-filters[0]=a", |Qs(v)| async move {
            assert!(v.decode);
            assert_eq!(v.pre_filters.len(), 1);
            assert_eq!(v.pre_filters[0], "a".to_string());
            assert!(v.post_filters.is_empty());
        })
        .await,
        StatusCode::OK
    );

    assert_eq!(
        do_test("pre-filters[0]=a&pre-filters[1]=b", |Qs(v)| async move {
            assert!(v.decode);
            assert_eq!(v.pre_filters.len(), 2);
            assert_eq!(v.pre_filters[0], "a".to_string());
            assert_eq!(v.pre_filters[1], "b".to_string());
            assert!(v.post_filters.is_empty());
        })
        .await,
        StatusCode::OK
    );

    assert_eq!(
        do_test("pre-filters[1]=a", |Qs(v)| async move {
            assert!(v.decode);
            assert_eq!(v.pre_filters.len(), 1);
            assert_eq!(v.pre_filters[0], "a".to_string());
            assert!(v.post_filters.is_empty());
        })
        .await,
        StatusCode::OK
    );

    assert_eq!(
        do_test("pre-filters[1]=a&pre-filters[2]=b", |Qs(v)| async move {
            assert!(v.decode);
            assert_eq!(v.pre_filters.len(), 2);
            assert_eq!(v.pre_filters[0], "a".to_string());
            assert_eq!(v.pre_filters[1], "b".to_string());
            assert!(v.post_filters.is_empty());
        })
        .await,
        StatusCode::OK
    );

    assert_eq!(
        do_test("pre-filters=a", |_| async move {}).await,
        StatusCode::BAD_REQUEST
    );

    assert_eq!(
        do_test("pre-filters[x]=a", |_| async move {}).await,
        StatusCode::BAD_REQUEST
    );

    assert_eq!(
        do_test("pre-filters[0]=a&pre-filters[0]=b", |_| async move {}).await,
        StatusCode::BAD_REQUEST
    );

    assert_eq!(
        do_test(
            "decode=1&pre-filters[]=a&post-filters[]=b",
            |Qs(v)| async move {
                assert!(v.decode);
                assert_eq!(v.pre_filters.len(), 1);
                assert_eq!(v.pre_filters[0], "a".to_string());
                assert_eq!(v.post_filters.len(), 1);
                assert_eq!(v.post_filters[0], "b".to_string());
            }
        )
        .await,
        StatusCode::OK
    );

    assert_eq!(
        do_test(
            "pre-filters[]=a&decode=1&post-filters[]=b",
            |Qs(v)| async move {
                assert!(v.decode);
                assert_eq!(v.pre_filters.len(), 1);
                assert_eq!(v.pre_filters[0], "a".to_string());
                assert_eq!(v.post_filters.len(), 1);
                assert_eq!(v.post_filters[0], "b".to_string());
            }
        )
        .await,
        StatusCode::OK
    );
}

async fn get_with_peer_addr(url: &str, addr: Option<SocketAddr>) -> TestResponse {
    let config = config_for_test();
    let app = build_app(&config)
        .layer(helper::ReplaceConnectInfoLayer::new(addr))
        .with_state(Arc::new(AppState {
            config,
            string_table: string_table_for_test(),
            tuner_manager: TunerManagerStub,
            epg: EpgStub,
            recording_manager: RecordingManagerStub,
            timeshift_manager: TimeshiftManagerStub,
            onair_manager: OnairProgramManagerStub,
        }));
    TestClient::new(app).get(url).send().await
}

async fn get(url: &str) -> TestResponse {
    let app = create_app();
    TestClient::new(app).get(url).send().await
}

async fn head(url: &str) -> TestResponse {
    let app = create_app();
    TestClient::new(app).head(url).send().await
}

async fn post<T>(url: &str, data: T) -> TestResponse
where
    T: serde::Serialize,
{
    let app = create_app();
    TestClient::new(app).post(url).json(&data).send().await
}

async fn delete(url: &str) -> TestResponse {
    let app = create_app();
    TestClient::new(app).delete(url).send().await
}

fn create_app() -> Router {
    let config = config_for_test();
    build_app(&config).with_state(Arc::new(AppState {
        config,
        string_table: string_table_for_test(),
        tuner_manager: TunerManagerStub,
        epg: EpgStub,
        recording_manager: RecordingManagerStub,
        timeshift_manager: TimeshiftManagerStub,
        onair_manager: OnairProgramManagerStub,
    }))
}

fn config_for_test() -> Arc<Config> {
    let config_yaml = format!(
        r#"
        server:
          mounts:
            /src:
              path: {manifest_dir}
            /src-with-listing:
              path: {manifest_dir}
              listing: true
            /src-with-index:
              path: {manifest_dir}
              index: Cargo.toml
            /Cargo.toml:
              path: {manifest_dir}/Cargo.toml
        # Disable service and program filters
        filters:
          service-filter:
            command: ''
          program-filter:
            command: ''
        # filters for testing
        pre-filters:
          cat:
            command: cat
        post-filters:
          cat:
            command: cat
          mp4:
            command: cat
            content-type: video/mp4
        recording:
          # Enable endpoints for recording
          basedir: /tmp
        # Enable endpoints for timeshift recording
        timeshift:
          recorders:
            test:
              service-id: 1
              ts-file: /dev/null
              data-file: /dev/null
              num-chunks: 100
        # logo for SID#1
        resource:
          logos:
            - service-id: 1
              image: /dev/null
        "#,
        manifest_dir = env!("CARGO_MANIFEST_DIR"),
    );

    Arc::new(
        serde_yaml::from_str::<Config>(&config_yaml)
            .unwrap()
            .normalize(),
    )
}

fn string_table_for_test() -> Arc<StringTable> {
    crate::string_table::load(
        format!("{}/../resources/strings.yml", env!("CARGO_MANIFEST_DIR")).as_str(),
    )
}

mod helper {
    use std::net::SocketAddr;
    use std::task::Context;
    use std::task::Poll;

    use axum::body::Body;
    use axum::extract::ConnectInfo;
    use axum::http::Request;
    use axum::response::Response;
    use futures::future::BoxFuture;
    use tower::Layer;
    use tower::Service;

    #[derive(Clone)]
    pub(super) struct ReplaceConnectInfoLayer(Option<SocketAddr>);

    impl ReplaceConnectInfoLayer {
        pub(super) fn new(addr: Option<SocketAddr>) -> Self {
            ReplaceConnectInfoLayer(addr)
        }
    }

    impl<S> Layer<S> for ReplaceConnectInfoLayer {
        type Service = ReplaceConnectInfoService<S>;

        fn layer(&self, inner: S) -> Self::Service {
            ReplaceConnectInfoService {
                inner,
                addr: self.0.clone(),
            }
        }
    }

    #[derive(Clone)]
    pub(super) struct ReplaceConnectInfoService<S> {
        inner: S,
        addr: Option<SocketAddr>,
    }

    impl<S> Service<Request<Body>> for ReplaceConnectInfoService<S>
    where
        S: Service<Request<Body>, Response = Response> + Send + 'static,
        S::Future: Send + 'static,
    {
        type Response = S::Response;
        type Error = S::Error;
        // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`
        type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

        fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
            self.inner.poll_ready(cx)
        }

        fn call(&mut self, mut req: Request<Body>) -> Self::Future {
            match self.addr {
                Some(addr) => {
                    req.extensions_mut().insert(ConnectInfo(addr.clone()));
                }
                None => {
                    req.extensions_mut().remove::<ConnectInfo<SocketAddr>>();
                }
            }
            let fut = self.inner.call(req);
            Box::pin(async move { Ok(fut.await?) })
        }
    }
}
// </coverage:exclude>
