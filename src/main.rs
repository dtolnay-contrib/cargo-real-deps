use cargo::core::compiler::{CompileKind, RustcTargetData};
use cargo::core::resolver::features::{ForceAllTargets, HasDevUnits, RequestedFeatures};
use cargo::core::resolver::ResolveOpts;
use cargo::core::{PackageIdSpec, Workspace};
use cargo::ops;
use cargo::util::command_prelude::{App, Arg};
use cargo::util::config::Config;
use cargo::util::errors::CargoResult;
use cargo::util::interning::InternedString;
use std::collections::BTreeSet;
use std::env;
use std::rc::Rc;

fn main() -> CargoResult<()> {
    let matches = App::new("cargo-real-deps")
        .arg(
            Arg::with_name("path")
                .short("p")
                .long("path")
                .required(true)
                .takes_value(true)
                .help("path to Cargo.toml"),
        )
        .arg(
            Arg::with_name("all-features")
                .long("all-features")
                .help("activate all features"),
        )
        .arg(
            Arg::with_name("no-default-features")
                .long("no-default-features")
                .help("deactivate default features"),
        )
        .arg(
            Arg::with_name("features")
                .long("features")
                .takes_value(true)
                .value_delimiter(",")
                .help("activates some features"),
        )
        .get_matches();

    let path = env::current_dir()
        .unwrap()
        .join(matches.value_of("path").unwrap());
    let all_features = matches.is_present("all-features");
    let uses_default_features = !matches.is_present("no-default-features");
    let features = matches
        .values_of("features")
        .map_or_else(BTreeSet::new, |v| v.map(InternedString::new).collect());

    let config = Config::default()?;
    let ws = Workspace::new(&path, &config)?;
    let requested_kinds = [CompileKind::Host];
    let target_data = RustcTargetData::new(&ws, &requested_kinds)?;
    let opts = ResolveOpts {
        dev_deps: false,
        features: RequestedFeatures {
            features: Rc::new(features),
            all_features,
            uses_default_features,
        },
    };
    let package_id = ws.current().unwrap().package_id();
    let specs = [PackageIdSpec::from_package_id(package_id)];
    let workspace_resolve = ops::resolve_ws_with_opts(
        &ws,
        &target_data,
        &requested_kinds,
        &opts,
        &specs,
        HasDevUnits::No,
        ForceAllTargets::No,
    )?;
    let resolve = workspace_resolve.targeted_resolve;

    let package_ids = resolve.sort();
    /*
    println!("metadata: {:?}", resolve.metadata());
    let packige = ws.current()?;
    println!("current package: {:?}", packige);
    println!("current id: {:?}", packige.package_id());
    //println!("summary: {:?}", packige.summary());
    //println!("targets: {:#?}", packige.targets());
    let members = ws.members().collect::<Vec<_>>();
    println!("workspace members: {:?}", members);
    */

    for id in package_ids {
        println!("{} {} {:?}", id.name(), id.version(), resolve.features(id));
    }

    Ok(())
}
