use criterion::{Criterion, black_box, criterion_group, criterion_main};

use ark::{
    ArkCommand, ArkConfig, ArkOutput, ArkOutputLine, InstallPlan, InstallStep, PackageDb,
    PackageDbEntry, TransactionLog, TransactionOp, TransactionOpStatus, TransactionOpType,
    parse_args,
};
use chrono::Utc;
use nous::PackageSource;

fn bench_parse_args(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse_args");

    group.bench_function("install_single", |b| {
        b.iter(|| parse_args(black_box(&["install", "nginx"])).unwrap());
    });

    group.bench_function("install_multiple_force", |b| {
        b.iter(|| {
            parse_args(black_box(&[
                "install", "--force", "nginx", "curl", "wget", "htop",
            ]))
            .unwrap()
        });
    });

    group.bench_function("search_with_source", |b| {
        b.iter(|| {
            parse_args(black_box(&[
                "search",
                "--source",
                "marketplace",
                "web",
                "server",
            ]))
            .unwrap()
        });
    });

    group.bench_function("remove_purge", |b| {
        b.iter(|| parse_args(black_box(&["remove", "--purge", "nginx"])).unwrap());
    });

    group.bench_function("list_filtered", |b| {
        b.iter(|| parse_args(black_box(&["list", "--marketplace"])).unwrap());
    });

    group.finish();
}

fn bench_ark_command_serde(c: &mut Criterion) {
    let mut group = c.benchmark_group("serde");

    let cmd = ArkCommand::Install {
        packages: vec!["nginx".into(), "curl".into(), "wget".into()],
        force: true,
    };
    let json = serde_json::to_string(&cmd).unwrap();

    group.bench_function("command_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&cmd)).unwrap());
    });

    group.bench_function("command_deserialize", |b| {
        b.iter(|| serde_json::from_str::<ArkCommand>(black_box(&json)).unwrap());
    });

    let config = ArkConfig::default();
    let config_json = serde_json::to_string(&config).unwrap();

    group.bench_function("config_serialize", |b| {
        b.iter(|| serde_json::to_string(black_box(&config)).unwrap());
    });

    group.bench_function("config_deserialize", |b| {
        b.iter(|| serde_json::from_str::<ArkConfig>(black_box(&config_json)).unwrap());
    });

    group.finish();
}

fn bench_package_db(c: &mut Criterion) {
    let mut group = c.benchmark_group("package_db");

    // Build a database with 100 entries
    let mut db = PackageDb::new();
    for i in 0..100 {
        db.register(PackageDbEntry {
            name: format!("package-{}", i),
            version: format!("{}.0.0", i),
            source: if i % 3 == 0 {
                PackageSource::Marketplace
            } else {
                PackageSource::System
            },
            installed_at: Utc::now(),
            installed_by: "root".into(),
            size_bytes: 1024 * (i + 1),
            checksum: format!("sha256-{:064x}", i),
            files: (0..5)
                .map(|f| format!("/usr/lib/package-{}/file-{}", i, f))
                .collect(),
            dependencies: if i > 0 {
                vec![format!("package-{}", i - 1)]
            } else {
                vec![]
            },
            transaction_id: Some(format!("txn-{:06}", i)),
            held: false,
        });
    }

    group.bench_function("search_100_packages", |b| {
        b.iter(|| db.search(black_box("package-5")));
    });

    group.bench_function("by_source_100_packages", |b| {
        b.iter(|| db.by_source(black_box(&PackageSource::Marketplace)));
    });

    group.bench_function("owner_of_100_packages", |b| {
        b.iter(|| db.owner_of(black_box("/usr/lib/package-50/file-2")));
    });

    group.bench_function("total_size_100_packages", |b| {
        b.iter(|| db.total_size());
    });

    group.bench_function("check_integrity_100_packages", |b| {
        b.iter(|| db.check_integrity());
    });

    group.finish();
}

fn bench_transaction_log(c: &mut Criterion) {
    let mut group = c.benchmark_group("transaction_log");

    group.bench_function("begin_commit_cycle", |b| {
        b.iter(|| {
            let mut log = TransactionLog::new();
            let id = log.begin("bench-user");
            log.add_op(
                &id,
                TransactionOp {
                    op_type: TransactionOpType::Install,
                    package: "nginx".into(),
                    version: Some("1.24".into()),
                    source: PackageSource::System,
                    status: TransactionOpStatus::Pending,
                    error: None,
                },
            );
            log.mark_op_complete(&id, "nginx");
            log.commit(&id);
            black_box(&log);
        });
    });

    // Benchmark with a pre-populated log
    let mut pre_log = TransactionLog::new();
    for i in 0..50 {
        let id = pre_log.begin(&format!("user-{}", i));
        pre_log.add_op(
            &id,
            TransactionOp {
                op_type: TransactionOpType::Install,
                package: format!("pkg-{}", i),
                version: Some("1.0".into()),
                source: PackageSource::System,
                status: TransactionOpStatus::Complete,
                error: None,
            },
        );
        pre_log.commit(&id);
    }

    group.bench_function("recent_from_50", |b| {
        b.iter(|| pre_log.recent(black_box(10)));
    });

    group.bench_function("get_from_50", |b| {
        b.iter(|| pre_log.get(black_box("txn-000025")));
    });

    group.finish();
}

fn bench_format_plan(c: &mut Criterion) {
    let plan = InstallPlan {
        steps: vec![
            InstallStep::SystemInstall {
                package: "nginx".into(),
                version: Some("1.24".into()),
            },
            InstallStep::MarketplaceInstall {
                package: "agent-monitor".into(),
                version: Some("2.0".into()),
            },
            InstallStep::FlutterInstall {
                package: "settings-app".into(),
                version: None,
            },
            InstallStep::SystemUpdate,
        ],
        requires_root: true,
        estimated_size_bytes: 1024 * 1024,
    };

    c.bench_function("format_plan_4_steps", |b| {
        b.iter(|| ark::ArkPackageManager::format_plan(black_box(&plan)));
    });
}

fn bench_output_display(c: &mut Criterion) {
    let mut output = ArkOutput::new();
    output
        .lines
        .push(ArkOutputLine::Header("Search results".into()));
    for i in 0..20 {
        output.lines.push(ArkOutputLine::Package {
            name: format!("package-{}", i),
            version: format!("{}.0", i),
            source: PackageSource::System,
            description: format!("Description for package {}", i),
        });
    }
    output.lines.push(ArkOutputLine::Separator);
    output.lines.push(ArkOutputLine::Info {
        key: "Total".into(),
        value: "20 results".into(),
    });

    c.bench_function("output_display_20_packages", |b| {
        b.iter(|| black_box(&output).to_display_string());
    });
}

criterion_group!(
    benches,
    bench_parse_args,
    bench_ark_command_serde,
    bench_package_db,
    bench_transaction_log,
    bench_format_plan,
    bench_output_display,
);
criterion_main!(benches);
