import os
from pathlib import Path

from aurorapulse.core.agent import AuroraAgent
from aurorapulse.core.settings import OLLAMA_MODEL
from aurorapulse.integrations.music.spotify import SpotifyAuthError


RESET = "\033[0m"
DIM = "\033[2m"
BOLD = "\033[1m"
ACCENT = "\033[38;5;215m"
MUTED = "\033[38;5;246m"


def _terminal_width() -> int:
    try:
        return os.get_terminal_size().columns
    except OSError:
        return 80


def _line(char: str = "\u2500") -> str:
    return char * min(_terminal_width(), 88)


def _print_banner() -> None:
    workspace = Path.cwd()
    print("\033[2J\033[H", end="")
    print(f"{ACCENT}{BOLD}  A U R O R A P U L S E{RESET}")
    print(f"{DIM}  local-first assistant shell{RESET}")
    print()
    print(f"{MUTED}  Model     {RESET}{OLLAMA_MODEL}")
    print(f"{MUTED}  Mode      {RESET}CLI")
    print(f"{MUTED}  Workspace {RESET}{workspace}")
    print()
    print(f"{MUTED}{_line()}{RESET}")
    print(f"{DIM}  Type a request, or 'quit' to exit.{RESET}")
    print()


def run_cli() -> None:
    _print_banner()

    try:
        agent = AuroraAgent()
    except SpotifyAuthError as exc:
        print(f"助手> 配置错误：{exc}")
        return

    while True:
        user_text = input(f"{ACCENT}>{RESET} ").strip()
        if not user_text:
            continue
        if user_text.lower() in {"quit", "exit"}:
            print("助手> 下次见。")
            break

        try:
            print(f"助手> {agent.handle(user_text)}")
        except Exception as exc:
            print(f"助手> 执行失败：{exc}")
