use anyhow::{anyhow, Error, Result};
use bevy_app::prelude::*;
use bevy_ecs::prelude::*;
use bevy_fallible::*;

struct ShouldFail(bool);
#[derive(Default)]
struct ErrorStorage(Vec<Error>);
struct CustomRes(String);

#[fallible_system]
fn system() -> Result<()> {
    println!("simple system");
    Ok(())
}

#[fallible_system]
fn system_with_args(res: Res<CustomRes>) -> Result<()> {
    println!("res: {}", res.0);
    Ok(())
}

#[fallible_system]
fn system_with_commands(_commands: &mut Commands, res: Res<CustomRes>) -> Result<()> {
    println!("res with commands: {}", res.0);
    Ok(())
}

fn faulty_query(res: &ShouldFail) -> Result<()> {
    if res.0 {
        Err(anyhow!("fail"))
    } else {
        Ok(())
    }
}

#[fallible_system]
fn faulty_system(mut res: ResMut<ShouldFail>) -> Result<()> {
    res.0 = !res.0;
    let val = faulty_query(&res)?;
    Ok(val)
}

fn counter_system(mut events: ResMut<Events<SystemErrorEvent>>, mut storage: ResMut<ErrorStorage>) {
    storage.0.extend(events.drain().map(|e| anyhow!(e.error)));
}

#[test]
fn fallible_systems_test() {
    App::build()
        .add_plugin(FallibleSystemPlugin)
        .add_resource::<CustomRes>(CustomRes("hello world".to_owned()))
        .add_resource::<ShouldFail>(ShouldFail(true))
        .add_resource::<ErrorStorage>(ErrorStorage::default())
        .add_system(system.system())
        .add_system(system_with_args.system())
        .add_system(system_with_commands.system())
        .add_system(faulty_system.system())
        .add_system(counter_system.system())
        .set_runner(|mut app| {
            for _ in 0..10 {
                app.update();
            }

            let errors: Vec<_> = app
                .resources
                .get::<ErrorStorage>()
                .unwrap()
                .0
                .iter()
                .map(|e| e.to_string())
                .collect();
            assert_eq!(errors.len(), 5);
            assert_eq!(errors, vec!["fail", "fail", "fail", "fail", "fail"]);
        })
        .run();
}
