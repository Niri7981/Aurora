from typing import Dict

from aurorapulse.integrations.llm.gemma4_ollama import Gemma4Planner
from aurorapulse.integrations.music.spotify import SpotifyController


def _render_track(track: dict) -> str:
    artists = ", ".join(artist["name"] for artist in track.get("artists", []))
    return f"{track.get('name', '未知歌曲')} - {artists or '未知艺人'}"


class AuroraAgent:
    def __init__(self) -> None:
        self.brain = Gemma4Planner()
        self.spotify = SpotifyController()

    def handle(self, user_text: str) -> str:
        decision: Dict[str, object] = self.brain.decide(user_text)
        action = str(decision.get("action", "none"))
        query = str(decision.get("query", "")).strip()
        reply = str(decision.get("reply", "好的"))
        volume = int(decision.get("volume", 0) or 0)

        if action == "play_track":
            if not query:
                return "我还没听清你想放哪首歌。"
            track = self.spotify.search_track(query)
            if not track:
                return f"没有找到“{query}”这首歌。"
            self.spotify.start_track(track["uri"])
            return f"已开始播放：{_render_track(track)}"

        if action == "play_artist":
            if not query:
                return "我还没听清你想听哪位歌手。"
            track = self.spotify.search_artist_top_track(query)
            if not track:
                return f"没有找到“{query}”相关的可播放内容。"
            self.spotify.start_track(track["uri"])
            return f"先给你放：{_render_track(track)}"

        if action == "pause":
            self.spotify.pause()
            return "已暂停播放"

        if action == "resume":
            self.spotify.resume()
            return "继续播放了"

        if action == "next":
            self.spotify.next_track()
            return "已切到下一首"

        if action == "previous":
            self.spotify.previous_track()
            return "已回到上一首"

        if action == "volume":
            self.spotify.set_volume(volume)
            return f"音量已调到 {max(0, min(100, volume))}%"

        return reply

