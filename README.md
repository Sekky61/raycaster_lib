# Zobrazení rozsáhlých volumetrických dat na CPU

## Obsah
1. [Základní informace](#základní-informace)
2. [Překlad a spouštění](#překlad-a-spouštění)
3. [Dokumentace](#dokumentace)

# Základní informace

Autor: Michal Majer

Datum: 19.4.2022

# Překlad a spouštění

Práce využívá jazyk *Rust* a nástroj *Cargo*.
Doporučený postup je popsán [zde](https://www.rust-lang.org/tools/install).

Projekt je (dne `2022-04-19`) přeložitelný na nejnovější stabilní verzi jazyka (`1.60.0`).

**Varování**: Výstup překladu (složka `target`) může přesáhnout `10GB`.

## Seznam závislostí

* `cmake`
* `fontconfig`
* `libxcb`

## Postup instalace všech závislostí

Následuje postup instalace všech závislostí potřebných pro přeložení a spuštění projektu.
Postup byl vyzkoušen na čisté instalaci [Ubuntu 20.04](https://releases.ubuntu.com/20.04/).

```
sudo apt update
sudo apt upgrade
```

```
sudo apt install cmake
```

```
sudo apt install fontconfig-config libfontconfig libfontconfig1-dev
```

```
sudo apt install libxcb1-dev libxcb-keysyms1-dev libpango1.0-dev \
libxcb-util0-dev libxcb-icccm4-dev libyajl-dev \
libstartup-notification0-dev libxcb-randr0-dev \
libev-dev libxcb-cursor-dev libxcb-xinerama0-dev \
libxcb-xkb-dev libxkbcommon-dev libxkbcommon-x11-dev \
autoconf libxcb-xrm0 libxcb-xrm-dev automake libxcb-shape0-dev libxcb-xfixes0-dev
```

## překlad

Překlad probíhá přes nástroj `cargo` (případně `rustc`).

Příkaz `cargo build --release` přeloží všechny součásti projektu s optimalizacemi (žádoucí).
Překlad je inkrementální a překládají se i závislosti.

Profil s maximální optimalizací (a pomalým překladem): `release-full`.
Překlad a spouštění s tímto profilem:
```
cargo build --profile release-full
```

## Spouštění

Projekt se skládá ze dvou aplikací, `vol_app` a `vol_gen`.

Kompilace a spuštění demo aplikace
```
cargo run --release --bin vol_app
```
nebo (vyšší optimalizace)
```
cargo run --profile release-full --bin vol_app
```
Přeložená aplikace se nachází v `target/release/vol_app`.

Kompilace a spuštění aplikace pro generování objemů
```
cargo run --release --bin vol_gen
```
Přeložená aplikace se nachází v `target/release/vol_gen`.
`vol_gen` přijímá argumenty přes příkazovou řádku.

Příklady spuštění `vol_gen`:
```
cargo run --release --bin vol_gen -- --dims=100,100,100 --generator solid --sample 42 --output-file volumes/100_solid.vol
```

# Dokumentace

Pro vygenerování dokumentace použijte příkaz
```
cargo doc --no-deps
```
Argumentem `--open` se dokumentace automaticky otevře v prohlížeči, argument `--document-private-items` vygeneruje dokumentaci i neveřejných částí kódu.
