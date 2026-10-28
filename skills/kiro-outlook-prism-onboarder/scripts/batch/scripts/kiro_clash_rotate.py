#!/usr/bin/env python3
"""Rotate Clash Party through aws-sg or Youtu dedicated Hong Kong nodes."""
import http.client
import ipaddress
import json
import os
import re
import socket
import subprocess
import sys
import urllib.parse


class Connection(http.client.HTTPConnection):
    def __init__(self, path):
        super().__init__('localhost', timeout=10)
        self.path = path

    def connect(self):
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.settimeout(self.timeout)
        self.sock.connect(self.path)


def select_group(proxies, group):
    candidates = ['PROXY', '悠兔'] if group == 'auto' else [group]
    for name in candidates:
        info = proxies.get(name, {})
        if info.get('type') != 'Selector':
            continue
        if name == '悠兔':
            nodes = [node for node in info.get('all', []) if '专线' in node and '香港' in node]
        else:
            nodes = [node for node in info.get('all', []) if 'aws-sg' in node.lower()]
        if len(nodes) >= 2:
            return name, info, nodes
    raise RuntimeError('Expected a selector with at least two aws-sg or Youtu dedicated Hong Kong nodes')


def exit_ip():
    try:
        result = subprocess.check_output(
            ['curl', '--silent', '--show-error', '--max-time', '6', '--proxy',
             os.environ.get('KIRO_PROXY', 'http://127.0.0.1:7890'), 'https://api.ipify.org'],
            text=True, stderr=subprocess.DEVNULL, timeout=7)
        return str(ipaddress.ip_address(result.strip()))
    except (subprocess.SubprocessError, ValueError):
        return None


def main():
    command, group = sys.argv[1:3]
    processes = subprocess.check_output(['ps', '-axo', 'command='], text=True, timeout=10)
    paths = [m.group(1) for line in processes.splitlines() if 'mihomo' in line
             for m in [re.search(r'-ext-ctl-unix\s+(\S+\.sock)', line)] if m]
    if len(set(paths)) != 1:
        raise RuntimeError('Expected one Clash Party controller socket')
    def request(method, route, body=None):
        connection = Connection(paths[0])
        try:
            connection.request(method, route, json.dumps(body) if body else None,
                               {'Content-Type': 'application/json'})
            response = connection.getresponse()
            data = response.read()
            if response.status not in (200, 204):
                raise RuntimeError('Clash controller HTTP ' + str(response.status))
            return json.loads(data) if data else None
        finally:
            connection.close()

    group, info, nodes = select_group(request('GET', '/proxies')['proxies'], group)
    route = '/proxies/' + urllib.parse.quote(group, safe='')
    if command == 'check':
        print(json.dumps({'group': group, 'current': info.get('now'), 'nodes': nodes}))
        return
    if command != 'next':
        raise RuntimeError('Expected check or next')
    current = info.get('now')
    index = nodes.index(current) if current in nodes else -1
    target = nodes[(index + 1) % len(nodes)]
    previous_ip = exit_ip()
    request('PUT', route, {'name': target})
    if request('GET', route).get('now') != target:
        raise RuntimeError('Clash selection read-back mismatch')
    current_ip = exit_ip()
    print(json.dumps({'group': group, 'previous': current, 'current': target,
                      'previousExitIp': previous_ip, 'currentExitIp': current_ip,
                      'exitIpChanged': previous_ip != current_ip if previous_ip and current_ip else None}))


if __name__ == '__main__':
    main()
