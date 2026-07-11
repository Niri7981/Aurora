import subprocess
import sys
import time

from aurorapulse.integrations.music.spotify import SpotifyController


def _render_track(track: dict) -> str:
    artists = ", ".join(artist["name"] for artist in track.get("artists", []))
    return f"{track.get('name', '未知歌曲')} - {artists or '未知艺人'}"


def _open_spotify_client() -> None:
    if sys.platform == "darwin":
        subprocess.run(["open", "-a", "Spotify"], check=False)
        time.sleep(2)


def play_artist(query: str) -> str:
    spotify = SpotifyController()
    _open_spotify_client()
    track = spotify.search_artist_top_track(query)
    if not track:
        raise RuntimeError(f"没有找到“{query}”相关的可播放内容。")
    spotify.start_track(track["uri"])
    return f"先给你放：{_render_track(track)}"


def play_track(query: str) -> str:
    spotify = SpotifyController()
    _open_spotify_client()
    track = spotify.search_track(query)
    if not track:
        raise RuntimeError(f"没有找到“{query}”这首歌。")
    spotify.start_track(track["uri"])
    return f"已开始播放：{_render_track(track)}"


def main() -> int:
    if len(sys.argv) < 3:
        print("usage: spotify_tool.py play_artist|play_track <query>", file=sys.stderr)
        return 2

    action = sys.argv[1]
    query = " ".join(sys.argv[2:]).strip()
    if not query:
        print("Spotify 查询不能为空。", file=sys.stderr)
        return 2

    try:
        if action == "play_artist":
            print(play_artist(query))
            return 0
        if action == "play_track":
            print(play_track(query))
            return 0
        print(f"unknown Spotify action: {action}", file=sys.stderr)
        return 2
    except Exception as exc:
        print(str(exc), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
