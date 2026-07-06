# Bevy WASM

Mod your Bevy games with WebAssembly!

[![CI](https://github.com/I-Ruiz/bevy_wasm_modding/actions/workflows/rust.yml/badge.svg?branch=main)](https://github.com/I-Ruiz/bevy_wasm_modding/actions)
[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT%2FApache-blue.svg)](https://github.com/I-Ruiz/bevy_wasm_modding#license)
[![Crates.io](https://img.shields.io/crates/d/bevy_wasm.svg?color=blue)](https://crates.io/crates/bevy_wasm_modding)<br/>
[![Bevy](https://img.shields.io/badge/bevy-v0.10-blueviolet)](https://crates.io/crates/bevy)

|                    |                                                                                                                                                                                            |               |
| ------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------- |
| `bevy_wasm_modding`        | [![](https://img.shields.io/crates/v/bevy_wasm_modding.svg)](https://crates.io/crates/bevy_wasm_modding) [![](https://docs.rs/bevy_wasm_modding/badge.svg)](https://docs.rs/bevy_wasm)                             | For games     |
| `bevy_wasm_modding_sys`    | [![](https://img.shields.io/crates/v/bevy_wasm_modding_sys.svg)](https://crates.io/crates/bevy_wasm_modding_sys) [![](https://docs.rs/bevy_wasm_modding_sys/badge.svg)](https://docs.rs/bevy_wasm_modding_sys)             | For mods      |
| `bevy_wasm_modding_shared` | [![](https://img.shields.io/crates/v/bevy_wasm_modding_shared.svg)](https://crates.io/crates/bevy_wasm_modding_shared) [![](https://docs.rs/bevy_wasm_modding_shared/badge.svg)](https://docs.rs/bevy_wasm_modding_shared) | For protocols |

See [examples/cubes](https://github.com/I-Ruiz/bevy_wasm_modding/tree/main/examples/cubes) for a comprehensive example of how to use this.

[Changelog](https://github.com/I-Ruiz/bevy_wasm_modding/blob/main/CHANGELOG.md)

## Protocol

Our protocol crate defines the two message types for communicating between the game and mods.

```toml
[dependencies]
bevy_wasm_modding_shared = "0.19.0"
serde = { version = "1.0", features = ["derive"] }
```

```rust
use bevy_wasm_modding_shared::prelude::*;
use serde::{Deserialize, Serialize};
use bevy_ecs::message::{MessageReader, MessageWriter, Message};

/// The version of the protocol. Automatically set from the `CARGO_PKG_VERSION` environment variable.
pub const PROTOCOL_VERSION: Version = version!();

/// A message to be sent Mod -> Game.
#[derive(Message, Clone, Serialize, Deserialize, Debug)]
pub enum ModMessage {
    Hello,
}

/// A message to be sent Game -> Mod.
#[derive(Message, Clone, Serialize, Deserialize, Debug)]
pub enum GameMessage {
    HiThere,
}
```

## Game

Our game will import `WasmPlugin` from [`bevy_wasm`](https://crates.io/crates/bevy_wasm_modding), and use it to automatically send and receive messages with the mods.

```toml
[dependencies]
bevy = "0.19.0"
bevy_wasm = "0.19.0"
my_game_protocol = { git = "https://github.com/username/my_game_protocol" }
```

```rust
use bevy::{log::LogPlugin, prelude::*};
use bevy_wasm_modding::prelude::*;
use simple_protocol::{GameMessage, ModMessage, PROTOCOL_VERSION};
use bevy::ecs::message::{MessageWriter, Message, MessageReader};

fn main() {
    App::new()
        .add_plugins(LogPlugin::default())
        .add_plugins(AssetPlugin::default())
        .add_plugins(MinimalPlugins)
        .add_plugins(WasmPlugin::<GameMessage, ModMessage>::new(PROTOCOL_VERSION))
        .add_systems(Startup, insert_mods)
        .add_systems(Update, listen_for_mod_messages)
        .add_systems(Update, send_messages_to_mods)
        .run();
}

fn insert_mods(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(WasmMod {
        wasm: asset_server.load("simple_mod.wasm"),
    });
}

fn listen_for_mod_messages(mut events: MessageReader<ModMessage>) {
    for event in events.read() {
        match event {
            ModMessage::Hello => {
                info!("The mod said hello!");
            }
        }
    }
}

fn send_messages_to_mods(mut events: MessageWriter<GameMessage>) {
    events.write(GameMessage::HiThere);
}
```

## Mod

Our mod will import `FFIPlugin` from [`bevy_wasm_sys`](https://crates.io/crates/bevy_wasm_modding_sys), and use it to automatically send and receive messages with the game.

```toml
[dependencies]
bevy_wasm_modding_sys = "0.19.0"
my_game_protocol = { git = "https://github.com/username/my_game_protocol" }
```

```rust
use bevy_wasm_modding_sys::prelude::*;
use simple_protocol::{GameMessage, ModMessage, PROTOCOL_VERSION};
use bevy_ecs::message::{MessageWriter, MessageReader};

#[allow(clippy::missing_safety_doc)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn build_app() {
    App::new()
        .add_plugins(FFIPlugin::<GameMessage, ModMessage>::new(PROTOCOL_VERSION))
        .add_systems(Update, listen_for_game_messages)
        .add_systems(Update, send_messages_to_game)
        .run();
}

fn listen_for_game_messages(mut events: MessageReader<GameMessage>) {
    for event in events.read() {
        match event {
            GameMessage::HiThere => {
                info!("The game said hi there!");
            }
        }
    }
}

fn send_messages_to_game(mut events: MessageWriter<ModMessage>) {
    events.write(ModMessage::Hello);
}
```

## Sharing Resources

**Protocol:**

```rust
#[derive(Debug, Default, Clone, Resource, Serialize, Deserialize, Reflect)]
pub struct MyCoolResource {
    pub value: u32,
    pub string: String,
}
```

**Game:**

```rust
App::new()
    ...
    .insert_resource(MyCoolResource {
        value: 0,
        string: "Hello from MyCoolResource!".to_string(),
    })
    .add_plugins(
        WasmPlugin::<HostMessage, ModMessage>::new(PROTOCOL_VERSION)
            .share_resource::<MyCoolResource>(),
    )
    .add_systems(Update, update_resource)
    ...

fn update_resource(mut my_cool_resource: ResMut<MyCoolResource>) {
    my_cool_resource.value += 1;
}
```

**Mod:**

```rust
App::new()
    ...
    .add_plugins(FFIPlugin::<HostMessage, ModMessage>::new(PROTOCOL_VERSION))
    .add_systems(Startup, startup_system)
    .add_systems(Update, print_resource_value)
    ...

fn startup_system(mut resources: ResMut<ExternResources>) {
    resources.insert::<MyCoolResource>();
}

fn print_resource_value(resource: ExternRes<MyCoolResource>) {
    info!("{:?}", resource);
}
```

See [examples/shared_resources](https://github.com/I-Ruiz/bevy_wasm_modding/tree/main/examples/shared_resources) for a full example.

## Roadmap

|     |                                                  |
| --- | ------------------------------------------------ |
| ✅  | wasmtime runtime in games                        |
| ✅  | Send messages from mods to game                  |
| ✅  | Send messages from game to mods                  |
| ✅  | Multi-mod support                                |
| ✅  | Time keeping                                     |
| ✅  | Protocol version checking                        |
| ✅  | Extern Resource                                  |
| ✅  | Startup system mod loading                       |
| ✅  | Direct update control                            |
| ✅  | Mod unloading                                    |
| ✅  | Mod discrimination (events aren't broadcast all) |
| ✅  | Browser support                                  |
| ⬜  | Extern Query                                     |
| ⬜  | Synced time                                      |
| ⬜  | Mod hotloading                                   |
| ⬜  | Automatic component syncing                      |

## License

Bevy WASM modding is free, open source and permissively licensed!
Except where noted (below and/or in individual files), all code in this repository is dual-licensed under either:

-   MIT License ([LICENSE-MIT](LICENSE-MIT) or [http://opensource.org/licenses/MIT](http://opensource.org/licenses/MIT))
-   Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or [http://www.apache.org/licenses/LICENSE-2.0](http://www.apache.org/licenses/LICENSE-2.0))

at your option.
This means you can select the license you prefer!
This dual-licensing approach is the de-facto standard in the Rust ecosystem and there are [very good reasons](https://github.com/bevyengine/bevy/issues/2373) to include both.

### Your contributions

Unless you explicitly state otherwise,
any contribution intentionally submitted for inclusion in the work by you,
as defined in the Apache-2.0 license,
shall be dual licensed as above,
without any additional terms or conditions.
