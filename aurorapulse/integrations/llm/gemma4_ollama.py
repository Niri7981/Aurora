import json
from typing import Any, Dict
from urllib import request

from aurorapulse.core.settings import OLLAMA_MODEL, OLLAMA_URL


SYSTEM_PROMPT = """你是 AuroraPulse 的本地大脑，运行在本机的 Gemma4 上。

你的职责是把用户自然语言转换成结构化的音乐控制意图。
你只能输出 JSON，不要输出解释。

格式：
{
  "action": "play_track|play_artist|pause|resume|next|previous|volume|none",
  "query": "歌曲名或艺人名，没有则为空字符串",
  "volume": 0,
  "reply": "给用户的一句简短中文回复"
}
"""


class Gemma4Planner:
    def decide(self, user_text: str) -> Dict[str, Any]:
        payload = {
            "model": OLLAMA_MODEL,
            "stream": False,
            "messages": [
                {"role": "system", "content": SYSTEM_PROMPT},
                {"role": "user", "content": user_text},
            ],
            "format": {
                "type": "object",
                "properties": {
                    "action": {"type": "string"},
                    "query": {"type": "string"},
                    "volume": {"type": "integer"},
                    "reply": {"type": "string"},
                },
                "required": ["action", "query", "volume", "reply"],
            },
            "options": {"temperature": 0.2},
        }
        body = json.dumps(payload).encode("utf-8")
        req = request.Request(
            f"{OLLAMA_URL}/api/chat",
            data=body,
            headers={"Content-Type": "application/json"},
            method="POST",
        )
        with request.urlopen(req, timeout=90) as resp:
            response = json.loads(resp.read().decode("utf-8"))
        content = response["message"]["content"]
        parsed = json.loads(content)
        parsed.setdefault("action", "none")
        parsed.setdefault("query", "")
        parsed.setdefault("volume", 0)
        parsed.setdefault("reply", "好的")
        return parsed

