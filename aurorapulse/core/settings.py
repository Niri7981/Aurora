import os
from pathlib import Path


PROJECT_ROOT = Path(__file__).resolve().parent.parent.parent
ENV_PATH = PROJECT_ROOT / ".env"
TOKEN_PATH = PROJECT_ROOT / "token.json"


def load_dotenv() -> None:
    if not ENV_PATH.exists():
        return

    for line in ENV_PATH.read_text(encoding="utf-8").splitlines():
        raw = line.strip()
        if not raw or raw.startswith("#") or "=" not in raw or raw.startswith("["):
            continue
        key, value = raw.split("=", 1)
        os.environ.setdefault(key.strip(), value.strip())


load_dotenv()


OLLAMA_MODEL = os.environ.get("OLLAMA_MODEL", "gemma4:e4b")
OLLAMA_URL = os.environ.get("OLLAMA_URL", "http://127.0.0.1:11434")
SPOTIFY_CLIENT_ID = os.environ.get("SPOTIFY_CLIENT_ID", "")
SPOTIFY_REDIRECT_URI = os.environ.get(
    "SPOTIFY_REDIRECT_URI", "http://127.0.0.1:8888/callback"
)
