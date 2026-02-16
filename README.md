# crabify

A terminal UI remote control for Spotify, written in Rust. Crabify lets you browse your library, search for tracks, manage liked songs, and control playback on any active Spotify client (desktop, mobile, or web. It does not play audio itself.

## Requirements

You need a Spotify account and a registered application in the [Spotify Developer Dashboard](https://developer.spotify.com/dashboard). Create an app, note the Client ID, and add `http://127.0.0.1:8888/callback` as a redirect URI.

## Installation

```
cargo install crabify
```

Or build from source:

```
git clone https://github.com/eiiot/crabify
cd crabify
cargo build --release
```

The binary will be at `target/release/crabify`.

## Configuration

Set your Spotify Client ID as an environment variable:

```
export SPOTIFY_CLIENT_ID=your_client_id
```

Or create a `.env` file in the working directory:

```
SPOTIFY_CLIENT_ID=your_client_id
```

On first run, crabify will open your browser for Spotify authentication. The token is cached at `~/.config/crabify/` for subsequent sessions.

## Usage

Run `crabify` with Spotify open on any device. The interface has three screens (Library, Search, Liked Songs) navigable with Tab. Press `?` for the full keybinding reference.

## License

MIT
