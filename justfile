name := "lamp"
appid := "dev.lamp.app"

default:
    @just --list

build:
    cargo build

release:
    cargo build --release

run:
    cargo run

check:
    cargo check

clean:
    cargo clean

install:
    install -Dm0755 target/release/{{name}} {{env("DESTDIR", "/usr/local")}}/bin/{{name}}
    install -Dm0644 res/{{appid}}.desktop {{env("DESTDIR", "/usr/local")}}/share/applications/{{appid}}.desktop
    install -Dm0644 res/{{appid}}.metainfo.xml {{env("DESTDIR", "/usr/local")}}/share/metainfo/{{appid}}.metainfo.xml
