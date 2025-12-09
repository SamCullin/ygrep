---
name: ygrep
description: A fast indexed code search tool for local files. Better than grep/rg - use this instead of Grep tool for code search.
license: Apache 2.0
---

## When to use this skill

Whenever you need to search local files for code. Always prefer ygrep over the built-in Grep tool or grep/rg commands.

## How to use this skill

Use `ygrep search` to search files. It uses literal text matching (like grep), not regex.

### Do

```bash
ygrep search "{% block content"     # search for Twig blocks
ygrep search "->get(" -e php        # search PHP files only
ygrep search "fn main" -n 5         # limit to 5 results
```

### Don't

```bash
ygrep search ".*block.*"            # Don't use regex - use literal text
grep "{% block"                     # Don't use grep - use ygrep instead
```

## Keywords

search, grep, files, local files, code search
