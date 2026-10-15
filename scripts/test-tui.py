#!/usr/bin/env python3
"""Exercise the real TUI in an isolated pseudo-terminal (Linux, stdlib only)."""
import fcntl
import os
import pty
import re
import select
import struct
import subprocess
import sys
import termios
import time

binary = sys.argv[1] if len(sys.argv) > 1 else "./target/release/cpuz-linux"
master, slave = pty.openpty()
original = termios.tcgetattr(slave)


def resize(columns, rows):
    fcntl.ioctl(slave, termios.TIOCSWINSZ, struct.pack("HHHH", rows, columns, 0, 0))


resize(100, 28)
process = subprocess.Popen(
    [binary, "--tui", "--seconds", "0.15", "--threads", "2"],
    stdin=slave, stdout=slave, stderr=slave,
    env={**os.environ, "TERM": "xterm-256color", "COLORTERM": "truecolor"},
)
transcript = bytearray()


def read_for(seconds):
    start = time.monotonic()
    while time.monotonic() - start < seconds:
        if select.select([master], [], [], 0.05)[0]:
            try:
                transcript.extend(os.read(master, 65536))
            except OSError:
                break


def screen_text():
    # Ratatui sends cursor deltas, so raw bytes are not a text screenshot.
    cells = [[" "] * 100 for _ in range(28)]
    x = y = 0
    text = transcript.decode("utf-8", "replace")
    for match in re.finditer(r"\x1b\[([0-?]*)([ -/]*)([@-~])|([^\x1b]+)", text):
        params, _, op, plain = match.groups()
        if plain is not None:
            for char in plain:
                if char == "\r":
                    x = 0
                elif char == "\n":
                    y = min(y + 1, 27)
                elif 0 <= x < 100 and 0 <= y < 28 and char.isprintable():
                    cells[y][x] = char
                    x += 1
            continue
        values = [int(v) if v else 1 for v in params.split(";")] if not params.startswith("?") else []
        if op in ("H", "f"):
            y = (values[0] if values else 1) - 1
            x = (values[1] if len(values) > 1 else 1) - 1
        elif op == "G":
            x = (values[0] if values else 1) - 1
        elif op == "A":
            y -= values[0]
        elif op == "B":
            y += values[0]
        elif op == "C":
            x += values[0]
        elif op == "D":
            x -= values[0]
        elif op == "J" and params in ("2", "3"):
            cells = [[" "] * 100 for _ in range(28)]
    return "\n".join("".join(row) for row in cells)


try:
    read_for(0.3)
    assert process.poll() is None, "TUI failed during startup"
    assert b"PROCESSOR" in transcript, "CPU panel missing"
    os.write(master, b"b")
    read_for(1.3)
    assert "Measurement complete" in screen_text(), "Both phases did not finish"
    resize(30, 10)
    # Crossterm's resize event is normally SIGWINCH-driven.
    import signal
    process.send_signal(signal.SIGWINCH)
    read_for(0.3)
    assert "Resize" in screen_text(), "Small terminal state missing"
    resize(100, 28)
    process.send_signal(signal.SIGWINCH)
    read_for(0.3)
    # Increase duration before starting/cancelling another multi run.
    os.write(master, b"]]]m")
    read_for(0.3)
    os.write(master, b"c")
    read_for(0.7)
    assert "Cancelled" in screen_text(), "Cancellation did not finish"
    os.write(master, b"q")
    process.wait(timeout=5)
    read_for(0.1)
    assert process.returncode == 0, process.returncode
    assert termios.tcgetattr(slave) == original, "Terminal settings not restored"
    print("PASS: both phases, CPU panel, resize, cancellation, exit and termios restoration")
finally:
    if process.poll() is None:
        process.kill()
        process.wait()
    with open("/tmp/cpuz-linux-tui-smoke.ansi", "wb") as output:
        output.write(transcript)
    os.close(master)
    os.close(slave)
