"""A tiny stand-in for llama-server used by supervisor tests.

Parses ``--host``/``--port`` like the real binary, then serves 200 on
``/health`` over loopback. No model, no inference — the supervisor contract
under test is process lifecycle + health polling, not real generation.
"""

from __future__ import annotations

import argparse
from http.server import BaseHTTPRequestHandler, HTTPServer


class _Handler(BaseHTTPRequestHandler):
    def do_GET(self) -> None:  # noqa: N802 - http.server API
        if self.path == "/health":
            self.send_response(200)
            self.end_headers()
            self.wfile.write(b'{"status":"ok"}')
        else:
            self.send_response(404)
            self.end_headers()

    def log_message(self, *_: object) -> None:  # silence test noise
        return


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("--host", default="127.0.0.1")
    parser.add_argument("--port", type=int, default=8737)
    args, _unknown = parser.parse_known_args()
    HTTPServer((args.host, args.port), _Handler).serve_forever()


if __name__ == "__main__":
    main()
