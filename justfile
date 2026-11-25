set dotenv-load := false

name := 'cosmic-bluetooth-gamepad'
appid := 'com.keewee.CosmicBluetoothGamepad'
prefix := '/usr'

_default:
    @just --list

build *args:
    cargo build --release --locked {{args}}

check:
    cargo fmt --check
    cargo clippy --locked --all-targets -- -D warnings
    cargo test --locked

test:
    cargo test --locked

run:
    cargo run --locked -- --standalone

install:
    install -Dm755 target/release/{{name}} {{prefix}}/bin/{{name}}
    install -Dm644 resources/{{appid}}.desktop {{prefix}}/share/applications/{{appid}}.desktop
    install -Dm644 resources/app.metainfo.xml {{prefix}}/share/metainfo/{{appid}}.metainfo.xml
    install -Dm644 resources/icon.svg {{prefix}}/share/icons/hicolor/scalable/apps/cosmic-bluetooth-gamepad.svg
    install -Dm644 LICENSE {{prefix}}/share/licenses/{{appid}}/LICENSE
    -update-desktop-database {{prefix}}/share/applications
    -gtk-update-icon-cache -f {{prefix}}/share/icons/hicolor

uninstall:
    rm -f {{prefix}}/bin/{{name}}
    rm -f {{prefix}}/share/applications/{{appid}}.desktop
    rm -f {{prefix}}/share/metainfo/{{appid}}.metainfo.xml
    rm -f {{prefix}}/share/icons/hicolor/scalable/apps/cosmic-bluetooth-gamepad.svg
    rm -f {{prefix}}/share/licenses/{{appid}}/LICENSE

clean:
    cargo clean
