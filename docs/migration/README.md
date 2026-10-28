# Migration maps

Mechanical renames a consumer applies when moving to a new major.

## 2.0.0: accessor names (ADR-009)

agnostik 2.0.0 renames its 706 field accessors from `<prefix>_<field>` to `<Struct>_<field>` /
`<Struct>_set_<field>`. Constructors, codecs and operations keep their names.

| 1.12.x | 2.0.0 |
|---|---|
| `amsg_content(m)` | `AgentMessage_content(m)` |
| `secctx_set_run_as_user(c, v)` | `SecurityContext_set_run_as_user(c, v)` |
| `cglim_memory_max(c)` | `CgroupLimits_memory_max(c)` |
| `agnostik_err_code(e)` | `AgnostikError_code(e)` |
| `rusage_memory(r)` | `ResourceUsage_mem_used(r)` (value structs take their JSON field names) |
| `span_set_parent(s, v)` | `Span_set_parent_id(s, v)` (one name per field) |

- [`2.0.0-renames.tsv`](2.0.0-renames.tsv) is the complete map: one `old<TAB>new` pair per line, 706 lines.
- `scripts/migrate-2.0.0.sh` in this repository applies it to a checkout:

```bash
bash /path/to/agnostik/scripts/migrate-2.0.0.sh --dry-run path/to/consumer
```

```bash
bash /path/to/agnostik/scripts/migrate-2.0.0.sh path/to/consumer
```

It rewrites the `.cyr`, `.tcyr` and `.bcyr` files under each directory, skipping that directory's own top-level
`lib/` and `build/` and any `agnostik.cyr`. Review the diff afterwards: an identifier of your own that equals an
old accessor name is renamed too. Then move `[deps.agnostik]` to the 2.0.0 tag and run `cyrius deps`.
