#!/usr/bin/env python3
"""Bound source reads/tool observations without models, hooks, or a persistent cache."""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys


def bounded(text, limit):
    if len(text) <= limit:
        return text
    head = limit // 2
    return text[:head] + '\n[clipped; retrieve the original by path]\n' + text[-(limit - head):]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest='action', required=True)
    read = commands.add_parser('read')
    read.add_argument('path', type=Path)
    read.add_argument('--start', type=int, default=1)
    read.add_argument('--lines', type=int, default=60)
    read.add_argument('--max-chars', type=int, default=4000)
    read.add_argument('--known-read-key')
    read.add_argument('--force', action='store_true')
    run = commands.add_parser('run')
    run.add_argument('--log', type=Path, required=True)
    run.add_argument('--max-chars', type=int, default=3000)
    run.add_argument('command', nargs=argparse.REMAINDER)
    args = parser.parse_args()
    if args.max_chars < 100:
        parser.error('--max-chars must be at least 100')
    if args.action == 'read':
        if args.start < 1 or args.lines < 1:
            parser.error('--start and --lines must be positive')
        data = args.path.read_bytes()
        digest = hashlib.sha256(data).hexdigest()
        descriptor = {'path': str(args.path.resolve()), 'sha256': digest, 'start': args.start, 'lines': args.lines, 'max_chars': args.max_chars}
        read_key = hashlib.sha256(json.dumps(descriptor, sort_keys=True).encode()).hexdigest()
        unchanged = args.known_read_key == read_key and not args.force
        print(json.dumps({'path': str(args.path.resolve()), 'sha256': digest,
                          'bytes': len(data), 'read_key': read_key, 'unchanged': unchanged,
                          'start': args.start, 'lines': args.lines}))
        if unchanged:
            print('Unchanged: reuse the matching prior excerpt only if still in active context; otherwise use --force.')
        else:
            text = data.decode('utf-8')
            excerpt = ''.join(text.splitlines(keepends=True)[args.start - 1:args.start - 1 + args.lines])
            print(bounded(excerpt, args.max_chars))
        return 0
    command = args.command[1:] if args.command[:1] == ['--'] else args.command
    if not command:
        parser.error('run requires a command after --')
    args.log.parent.mkdir(parents=True, exist_ok=True)
    # Do not overwrite or follow an existing log/symlink.
    with args.log.open('xb') as log:
        os.chmod(args.log, 0o600)
        try:
            result = subprocess.run(command, stdout=log, stderr=subprocess.STDOUT, check=False)
            status = result.returncode
        except OSError as error:
            log.write((str(error) + '\n').encode())
            status = 127
    data = args.log.read_bytes()
    print(json.dumps({'log': str(args.log.resolve()), 'bytes': len(data),
                      'sha256': hashlib.sha256(data).hexdigest(), 'exit_status': status}))
    print(bounded(data.decode('utf-8', errors='replace'), args.max_chars))
    return status if status >= 0 else min(255, 128 - status)


if __name__ == '__main__':
    raise SystemExit(main())
