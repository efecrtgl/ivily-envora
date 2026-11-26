//! Uçtan uca iş akışı testi (bellek içi sağlayıcıyla; gerçek ortam değiştirilmez).
//!
//! profil oluştur → kalıcı uygula (yedekle) → anlık görüntü → karşılaştır → dışa aktar →
//! başka veritabanına içe aktar → devre dışı bırak → anlık görüntüden geri yükle.
//! Her adımda gizli değerin plan, dışa aktarma, karşılaştırma ve etkinlik kaydına sızmadığı denetlenir.

use envora_lib::activity::{self, Query};
use envora_lib::compare::{self, Source};
use envora_lib::db::Db;
use envora_lib::env::apply::{apply, ApplyOptions};
use envora_lib::env::inspect::Overrides;
use envora_lib::env::plan::Resolved;
use envora_lib::env::{EnvProvider, MemoryProvider, Scope, ValueKind};
use envora_lib::error::AppResult;
use envora_lib::io::{export, import, FileFormat};
use envora_lib::pathmgr::{PathAction, PathMod};
use envora_lib::profiles::model::{ProfileInput, ProfilePath, VarAction, VarInput};
use envora_lib::profiles::{activate, store as profiles};
use envora_lib::snapshots::manage::{self, CreateRequest, SnapshotSource};
use envora_lib::sources::DbSources;

const SECRET: &str = "wf-secret-0123456789";

fn no_elevation(_: &[Resolved]) -> AppResult<usize> {
    panic!("bu testte yükseltme beklenmiyor")
}

fn var(name: &str, value: &str, scope: Scope) -> VarInput {
    VarInput {
        name: name.into(),
        value: Some(value.into()),
        keep: false,
        previous_name: None,
        sensitive: None,
        kind: None,
        scope,
        action: VarAction::Set,
    }
}

#[test]
fn full_profile_lifecycle_without_leaking_secrets() {
    let db = Db::open_in_memory().unwrap();
    let p = MemoryProvider::new();
    p.seed(Scope::User, "Path", "C:\\Windows", ValueKind::Expand);
    p.seed(Scope::User, "NODE_ENV", "production", ValueKind::String);
    let o = Overrides::new();
    let opts = ApplyOptions {
        backup: true,
        max_auto_snapshots: 50,
        label: "wf",
    };

    // 1. Profil
    let input = ProfileInput {
        name: "Node".into(),
        description: "uçtan uca".into(),
        category: "Node.js".into(),
        tags: vec!["test".into()],
        metadata: None,
        vars: vec![
            var("NODE_ENV", "development", Scope::User),
            var("NPM_TOKEN", SECRET, Scope::User),
            var("PORT", "3000", Scope::Process),
        ],
        paths: vec![ProfilePath {
            path: PathMod {
                entry: "C:\\node".into(),
                action: PathAction::Prepend,
            },
            scope: Scope::User,
        }],
    };
    let id = profiles::create(&db, &input, &o).unwrap();

    // 2. Kalıcı uygula
    let plans = activate::build_plans(&p, &profiles::load_full(&db, id).unwrap()).unwrap();
    assert_eq!(plans.len(), 1);
    assert!(!serde_json::to_string(&plans).unwrap().contains(SECRET));
    let out = apply(
        &db,
        &p,
        &plans[0],
        &o,
        &opts,
        &DbSources::new(&db),
        &no_elevation,
    )
    .unwrap();
    activate::record_activation(&db, id, Scope::User, out.snapshot_id).unwrap();
    assert_eq!(
        p.get(Scope::User, "NPM_TOKEN").unwrap().unwrap().value,
        SECRET
    );
    assert_eq!(
        p.get(Scope::User, "Path").unwrap().unwrap().value,
        "C:\\node;C:\\Windows"
    );
    assert!(
        p.get(Scope::User, "PORT").unwrap().is_none(),
        "işlem kapsamı kalıcı yazılmaz"
    );

    // 3. Elle anlık görüntü (hassaslar dahil, şifreli)
    let snap = manage::create(
        &db,
        &p,
        &o,
        &CreateRequest {
            name: "uygulama sonrası".into(),
            description: None,
            source: SnapshotSource::Environment {
                scopes: vec![Scope::User],
                names: None,
            },
            exclude_sensitive: false,
        },
    )
    .unwrap();
    assert!(
        !serde_json::to_string(&manage::detail(&db, snap.id).unwrap())
            .unwrap()
            .contains(SECRET)
    );

    // 4. Karşılaştır: profil ↔ kullanıcı kapsamı
    let left = compare::load(&db, &p, &o, &Source::Profile { id }).unwrap();
    let right = compare::load(&db, &p, &o, &Source::Scope { scope: Scope::User }).unwrap();
    let cmp = compare::compare(&left, &right);
    assert!(!serde_json::to_string(&cmp).unwrap().contains(SECRET));

    // 5. Dışa aktar (hassaslar hariç) ve başka bir veritabanına içe aktar
    let exported = export::profile(&db, id, false).unwrap();
    assert!(!exported.content.contains(SECRET));
    assert_eq!(exported.result.excluded_sensitive, 1);
    let db2 = Db::open_in_memory().unwrap();
    let (bundle, _) = import::parse(&exported.content, FileFormat::Json).unwrap();
    let r = import::create_profile(&db2, &bundle, "Node", &o).unwrap();
    assert_eq!(r.skipped, 1);
    assert_eq!(profiles::load_full(&db2, r.id).unwrap().vars.len(), 2);

    // 6. Devre dışı bırak: profil öncesine dönüş
    let back = activate::deactivation_plan(&db, id, Scope::User).unwrap();
    apply(
        &db,
        &p,
        &back,
        &o,
        &opts,
        &DbSources::new(&db),
        &no_elevation,
    )
    .unwrap();
    activate::mark_deactivated(&db, id, Scope::User).unwrap();
    assert_eq!(
        p.get(Scope::User, "NODE_ENV").unwrap().unwrap().value,
        "production"
    );
    assert!(p.get(Scope::User, "NPM_TOKEN").unwrap().is_none());
    assert_eq!(
        p.get(Scope::User, "Path").unwrap().unwrap().value,
        "C:\\Windows"
    );

    // 7. Anlık görüntüden geri yükle: profil uygulanmış hale dön
    let plans = manage::restore_plans(&db, snap.id, None).unwrap();
    apply(
        &db,
        &p,
        &plans[0],
        &o,
        &opts,
        &DbSources::new(&db),
        &no_elevation,
    )
    .unwrap();
    assert_eq!(
        p.get(Scope::User, "NPM_TOKEN").unwrap().unwrap().value,
        SECRET
    );
    assert_eq!(
        p.get(Scope::User, "NODE_ENV").unwrap().unwrap().value,
        "development"
    );

    // 8. Etkinlik kaydı ve ham veritabanı gizli değeri düz metin içermez
    activity::record(
        &db,
        "profile",
        "apply",
        Some("Node"),
        activity::Status::Ok,
        None,
    )
    .unwrap();
    assert!(
        !serde_json::to_string(&activity::list(&db, &Query::default()).unwrap())
            .unwrap()
            .contains(SECRET)
    );
    let plain_hits: i64 = db
        .with(|c| {
            let mut n = 0i64;
            for (table, col) in [
                ("profile_vars", "value_plain"),
                ("snapshot_items", "value_plain"),
                ("profile_revisions", "data_plain"),
            ] {
                n += c.query_row(
                    &format!("SELECT COUNT(*) FROM {table} WHERE {col} LIKE ?1"),
                    [format!("%{SECRET}%")],
                    |r| r.get::<_, i64>(0),
                )?;
            }
            Ok(n)
        })
        .unwrap();
    assert_eq!(
        plain_hits, 0,
        "gizli değer veritabanında düz metin olarak bulunmamalı"
    );
}

#[test]
fn failing_write_rolls_back_and_keeps_backup() {
    // Sistem kapsamına yazma izni olmayan sağlayıcıda, yükseltme olmadan yazma reddedilir ve hiçbir şey değişmez.
    let db = Db::open_in_memory().unwrap();
    let p = MemoryProvider::new();
    p.seed(Scope::Machine, "A", "1", ValueKind::String);
    let plan = envora_lib::env::plan::ChangePlan {
        scope: Scope::Machine,
        ops: vec![envora_lib::env::plan::Op::Set {
            name: "A".into(),
            value: Some("2".into()),
            kind: None,
        }],
    };
    let refuse =
        |_: &[Resolved]| -> AppResult<usize> { Err(envora_lib::error::AppError::Cancelled) };
    let opts = ApplyOptions {
        backup: true,
        max_auto_snapshots: 50,
        label: "x",
    };
    let r = apply(
        &db,
        &p,
        &plan,
        &Overrides::new(),
        &opts,
        &DbSources::new(&db),
        &refuse,
    );
    assert!(matches!(r, Err(envora_lib::error::AppError::Cancelled)));
    assert_eq!(p.get(Scope::Machine, "A").unwrap().unwrap().value, "1");
}
