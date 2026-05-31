import base64
import hashlib
import json
import secrets
import threading
import webbrowser
from http.server import BaseHTTPRequestHandler, HTTPServer
from typing import Any, Dict, List, Optional
from urllib import error, parse, request

from aurorapulse.core.settings import (
    SPOTIFY_CLIENT_ID,
    SPOTIFY_REDIRECT_URI,
    TOKEN_PATH,
)


AUTH_URL = "https://accounts.spotify.com/authorize"
TOKEN_URL = "https://accounts.spotify.com/api/token"
API_BASE = "https://api.spotify.com/v1"
SCOPES = [
    "user-read-playback-state",
    "user-modify-playback-state",
]


class SpotifyAuthError(RuntimeError):
    pass


def _b64url_sha256(value: str) -> str:
    digest = hashlib.sha256(value.encode("utf-8")).digest()
    return base64.urlsafe_b64encode(digest).rstrip(b"=").decode("utf-8")


def _read_token_cache() -> Optional[Dict[str, Any]]:
    if not TOKEN_PATH.exists():
        return None
    return json.loads(TOKEN_PATH.read_text(encoding="utf-8"))


def _write_token_cache(token_data: Dict[str, Any]) -> None:
    TOKEN_PATH.write_text(
        json.dumps(token_data, ensure_ascii=False, indent=2), encoding="utf-8"
    )


class _CallbackHandler(BaseHTTPRequestHandler):
    auth_code = None

    def do_GET(self) -> None:
        parsed = parse.urlparse(self.path)
        params = parse.parse_qs(parsed.query)
        _CallbackHandler.auth_code = params.get("code", [None])[0]

        body = (
            "<html><body><h2>Spotify 授权成功</h2>"
            "<p>可以回到终端继续了。</p></body></html>"
        )
        self.send_response(200)
        self.send_header("Content-Type", "text/html; charset=utf-8")
        self.end_headers()
        self.wfile.write(body.encode("utf-8"))

    def log_message(self, fmt: str, *args: Any) -> None:
        return


class SpotifyController:
    def __init__(self) -> None:
        if not SPOTIFY_CLIENT_ID or SPOTIFY_CLIENT_ID == "your_spotify_client_id":
            raise SpotifyAuthError("请先在 .env 里配置 SPOTIFY_CLIENT_ID")
        self.client_id = SPOTIFY_CLIENT_ID
        self.redirect_uri = SPOTIFY_REDIRECT_URI
        self.token_data = _read_token_cache()
        self._cached_country: Optional[str] = None

    def ensure_token(self) -> None:
        if self.token_data and self.token_data.get("access_token"):
            return
        self.login()

    def login(self) -> None:
        verifier = secrets.token_urlsafe(64)
        challenge = _b64url_sha256(verifier)
        auth_code = self._run_pkce_flow(challenge)
        self.token_data = self._exchange_code(auth_code, verifier)
        _write_token_cache(self.token_data)

    def _run_pkce_flow(self, challenge: str) -> str:
        parsed = parse.urlparse(self.redirect_uri)
        host = parsed.hostname or "127.0.0.1"
        port = parsed.port or 8888

        _CallbackHandler.auth_code = None
        server = HTTPServer((host, port), _CallbackHandler)
        thread = threading.Thread(target=server.handle_request, daemon=True)
        thread.start()

        query = parse.urlencode(
            {
                "client_id": self.client_id,
                "response_type": "code",
                "redirect_uri": self.redirect_uri,
                "scope": " ".join(SCOPES),
                "code_challenge_method": "S256",
                "code_challenge": challenge,
            }
        )
        webbrowser.open(f"{AUTH_URL}?{query}")
        thread.join(timeout=180)
        server.server_close()

        if not _CallbackHandler.auth_code:
            raise SpotifyAuthError("Spotify 授权超时，请重试")
        return _CallbackHandler.auth_code

    def _exchange_code(self, code: str, verifier: str) -> Dict[str, Any]:
        payload = parse.urlencode(
            {
                "client_id": self.client_id,
                "grant_type": "authorization_code",
                "code": code,
                "redirect_uri": self.redirect_uri,
                "code_verifier": verifier,
            }
        ).encode("utf-8")
        req = request.Request(
            TOKEN_URL,
            data=payload,
            headers={"Content-Type": "application/x-www-form-urlencoded"},
            method="POST",
        )
        with request.urlopen(req, timeout=30) as resp:
            return json.loads(resp.read().decode("utf-8"))

    def _refresh_token(self) -> None:
        refresh_token = (self.token_data or {}).get("refresh_token")
        if not refresh_token:
            self.login()
            return
        payload = parse.urlencode(
            {
                "client_id": self.client_id,
                "grant_type": "refresh_token",
                "refresh_token": refresh_token,
            }
        ).encode("utf-8")
        req = request.Request(
            TOKEN_URL,
            data=payload,
            headers={"Content-Type": "application/x-www-form-urlencoded"},
            method="POST",
        )
        with request.urlopen(req, timeout=30) as resp:
            refreshed = json.loads(resp.read().decode("utf-8"))
        refreshed["refresh_token"] = refreshed.get("refresh_token", refresh_token)
        self.token_data = refreshed
        _write_token_cache(self.token_data)

    def _request(
        self,
        method: str,
        path: str,
        *,
        params: Optional[Dict[str, Any]] = None,
        body: Optional[Dict[str, Any]] = None,
        allow_retry: bool = True,
    ) -> Any:
        self.ensure_token()
        query = f"?{parse.urlencode(params)}" if params else ""
        url = f"{API_BASE}{path}{query}"
        data = None if body is None else json.dumps(body).encode("utf-8")
        req = request.Request(
            url,
            data=data,
            headers={
                "Authorization": f"Bearer {self.token_data['access_token']}",
                "Content-Type": "application/json",
            },
            method=method,
        )
        try:
            with request.urlopen(req, timeout=30) as resp:
                payload = resp.read().decode("utf-8")
                if not payload:
                    return None
                return json.loads(payload)
        except error.HTTPError as exc:
            if exc.code == 401 and allow_retry:
                self._refresh_token()
                return self._request(
                    method, path, params=params, body=body, allow_retry=False
                )
            detail = exc.read().decode("utf-8", errors="ignore")
            raise RuntimeError(f"Spotify API 错误 {exc.code}: {detail}") from exc

    def get_user_country(self) -> str:
        if self._cached_country:
            return self._cached_country
        profile = self._request("GET", "/me")
        self._cached_country = profile.get("country") or "US"
        return self._cached_country

    def get_devices(self) -> List[Dict[str, Any]]:
        result = self._request("GET", "/me/player/devices")
        return result.get("devices", [])

    def pick_device_id(self) -> Optional[str]:
        devices = self.get_devices()
        if not devices:
            return None
        active = next((d for d in devices if d.get("is_active")), None)
        return (active or devices[0]).get("id")

    def search_track(self, query: str) -> Optional[Dict[str, Any]]:
        result = self._request(
            "GET",
            "/search",
            params={
                "q": query,
                "type": "track",
                "limit": 1,
                "market": self.get_user_country(),
            },
        )
        items = result.get("tracks", {}).get("items", [])
        return items[0] if items else None

    def search_artist_top_track(self, artist_name: str) -> Optional[Dict[str, Any]]:
        result = self._request(
            "GET",
            "/search",
            params={
                "q": artist_name,
                "type": "artist",
                "limit": 1,
                "market": self.get_user_country(),
            },
        )
        artists = result.get("artists", {}).get("items", [])
        if not artists:
            return None
        artist_id = artists[0]["id"]
        top = self._request(
            "GET",
            f"/artists/{artist_id}/top-tracks",
            params={"market": self.get_user_country()},
        )
        tracks = top.get("tracks", [])
        return tracks[0] if tracks else None

    def start_track(self, track_uri: str) -> None:
        device_id = self.pick_device_id()
        if not device_id:
            raise RuntimeError("没有可用的 Spotify 播放设备，请先打开 Spotify 客户端")
        self._request(
            "PUT",
            "/me/player/play",
            params={"device_id": device_id},
            body={"uris": [track_uri]},
        )

    def pause(self) -> None:
        self._request("PUT", "/me/player/pause")

    def resume(self) -> None:
        device_id = self.pick_device_id()
        if device_id:
            self._request(
                "PUT", "/me/player/play", params={"device_id": device_id}, body={}
            )
            return
        self._request("PUT", "/me/player/play", body={})

    def next_track(self) -> None:
        self._request("POST", "/me/player/next")

    def previous_track(self) -> None:
        self._request("POST", "/me/player/previous")

    def set_volume(self, volume_percent: int) -> None:
        volume_percent = max(0, min(100, volume_percent))
        self._request(
            "PUT", "/me/player/volume", params={"volume_percent": volume_percent}
        )

