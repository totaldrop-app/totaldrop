# Security policy

TotalDrop's whole purpose is to keep files private, so security reports get priority over everything else.

## Reporting a vulnerability

- Use GitHub's private vulnerability reporting on this repository ("Report a vulnerability" under the Security tab).
- Or email **security@totaldrop.app**.
- Please do not open a public issue, discussion or pull request for a vulnerability.

Include what you found, how to reproduce it, and what an attacker could do with it. You will get a reply within 72 hours.

## What we promise

- We will not take legal action against good-faith research that respects users' privacy and does not degrade the service.
- We will credit you in the advisory, unless you prefer to stay anonymous.
- We will publish an advisory for every confirmed vulnerability once a fix is released.

## How we handle a report

1. **Acknowledge** within 72 hours, privately, on the channel you used.
2. **Triage:** reproduce it, judge its impact against the [threat model](docs/tehdit-modeli.md), and agree a severity with you.
3. **Fix** in a private fork or a draft security advisory, with a test that fails before the fix. Fixes for exploitable issues come before any other work.
4. **Release** the fix for every affected platform, then publish a GitHub security advisory (with a CVE where one applies) and a Security entry in [CHANGELOG.md](CHANGELOG.md).
5. **Credit** the reporter in the advisory and the changelog, unless they ask to stay anonymous.

We aim to release a fix within 60 days of the report, and sooner for critical issues. If that is not possible, we tell you why and agree a new date.

## What you can expect from TotalDrop

TotalDrop is meant to:

- keep the content, names and folder structure of the files you send readable only by the device you send them to: not by our servers, Cloudflare or anyone on the network;
- let only the person who has the code receive the files, give an attacker one guess per transfer, and show both people four words to compare;
- protect today's transfers against someone who records them to break later with a quantum computer (ML-KEM-768);
- never write outside the folder the receiver chose, and never overwrite an existing file;
- work without an account, email address or phone number, and keep no logs on the servers.

TotalDrop does not:

- hide that a transfer happened, when, or roughly how big it was;
- hide your IP address from the servers, or from the other device unless you turn on the privacy setting;
- protect files on a device that is already compromised, or check whether a file you receive is harmful;
- have an independent security audit yet.

The [threat model](docs/tehdit-modeli.md) (in Turkish) lists every threat, how it is countered and what risk remains.

## Scope

In scope: the protocol specification, everything in this repository, and the project's servers and website at totaldrop.app.

Out of scope: denial-of-service by traffic volume, social engineering, and findings that need an already compromised device.

## Status

TotalDrop has not had an independent security audit yet. Version 1.0 will only be released after one.
