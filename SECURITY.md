# Security Policy

## Supported Versions
| Version  | Supported |
|----------|-----------|
| 0.97.x   | Yes       |
| 0.96.x   | Yes       |
| < 0.96.0 | No        |

## Scope
Agnostik is a type library with zero external dependencies. Primary attack surface: deserialization of untrusted input (`_from_str` parse functions, future `_from_json`). All types validate on construction where applicable. All parse functions return `Result` with descriptive error messages.

## Reporting a Vulnerability
Report security issues to the maintainer via GitHub private vulnerability reporting on the [agnostik repository](https://github.com/MacCracken/agnostik). Do not open public issues for security vulnerabilities.

Include:
- Description of the vulnerability
- Steps to reproduce
- Affected version(s)
- Impact assessment

You should receive a response within 48 hours.
